use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::game_history::GameHistory;
use super::lessons::Lessons;
use super::marker::Marker;
use super::payload::AllGameData;
use super::pending_answer::PendingAnswer;
use super::recap::Recap;
use super::recap_news::RecapNews;
use super::recap_prompt::{input, INSTRUCTIONS};
use super::recap_watch::RecapWatch;
use super::recap_window::RecapWindow;
use super::shown_benchmarks::ShownBenchmarks;
use super::snapshot::Snapshot;
use crate::advice::{Advisor, Question};
use crate::game_inputs::{ClientPhase, GameInputs};

// No player waits on a Recap's Lessons, so they get far longer than any Trigger's.
const LESSONS_DEADLINE: Duration = Duration::from_secs(30);
// Up to three Lessons take far longer to write than one Trigger's Advice, and 20 s
// still leaves the deadline room for a retry.
const LESSONS_ATTEMPT: Duration = Duration::from_secs(20);
const ASKER: &str = "lessons";

/// The last game's Recap: made as the game ends, its Lessons asked for, and its window
/// opened once the League client reaches the post-game screens.
pub(super) struct GameRecap {
    advisor: Advisor,
    history: GameHistory,
    watch: Option<RecapWatch>,
    recap: Option<Recap>,
    lessons: PendingAnswer<Lessons>,
    /// Whether the Recap changed since the app was last told.
    changed: bool,
    window: RecapWindow,
}

impl GameRecap {
    pub(super) fn new(advisor: &Advisor, history: &GameHistory) -> Self {
        Self {
            advisor: advisor.clone(),
            history: history.clone(),
            watch: None,
            recap: None,
            lessons: PendingAnswer::default(),
            changed: false,
            window: RecapWindow::Shut,
        }
    }

    /// Keeps the last game's Recap, whose Lessons may still be on their way.
    pub(super) fn start(&mut self, inputs: Arc<GameInputs>) {
        self.watch = Some(RecapWatch::start(inputs));
        self.window = RecapWindow::Shut;
    }

    pub(super) fn observe(
        &mut self,
        game_time: f64,
        snapshot: &Snapshot,
        benchmarks: &ShownBenchmarks,
        cs_pace: Option<Marker>,
    ) {
        if let Some(watch) = &mut self.watch {
            watch.observe(game_time, snapshot, benchmarks, cs_pace);
        }
    }

    pub(super) fn keep(&mut self, data: AllGameData) {
        if let Some(watch) = &mut self.watch {
            watch.keep(data);
        }
    }

    /// Makes the Recap from a payload holding `GameEnd`; once made, later ones do nothing.
    pub(super) fn game_ended(&mut self, data: &AllGameData) {
        if let Some(watch) = self.watch.take() {
            let ended = if data.active_player().is_some() {
                Some(data)
            } else {
                watch.last()
            };
            if let Some(ended) = ended {
                self.finish(&watch, ended);
            }
        }
    }

    /// Asks the League client whether the game is over, which makes the Recap from the
    /// last running poll, and whether the post-game screens show, which opens it.
    pub(super) async fn live_client_gone(&mut self, http: &reqwest::Client, lockfile: &Path) {
        if self.watch.is_none() && self.window != RecapWindow::Awaiting {
            return;
        }
        let phase = ClientPhase::read(http, lockfile).await;
        if phase.game_over() {
            if let Some(watch) = self.watch.take() {
                if let Some(last) = watch.last() {
                    self.finish(&watch, last);
                }
            }
        }
        if self.window == RecapWindow::Awaiting && phase == ClientPhase::PostGame {
            self.window = RecapWindow::Due;
        }
    }

    pub(super) async fn news(&mut self) -> RecapNews {
        if let Some(lessons) = self.lessons.finished().await {
            if let Some(recap) = &mut self.recap {
                recap.lessons = lessons;
                self.changed = true;
            }
        }
        let changed = std::mem::take(&mut self.changed);
        let open = self.window == RecapWindow::Due;
        if open {
            self.window = RecapWindow::Opened;
        }
        RecapNews {
            recap: self.recap.clone().filter(|_| changed),
            open,
        }
    }

    fn finish(&mut self, watch: &RecapWatch, data: &AllGameData) {
        let Some(mut recap) = watch.recap(data) else {
            return;
        };
        if let Some(champion) = data.active_player().and_then(|p| p.champion_id()) {
            recap.trend = self.history.trend_with(champion, watch.at_ten());
        }
        recap.lessons = match self.advisor.lessons_call() {
            Ok(call) => {
                let input = input(call.language, &recap, data, watch.tier());
                let advisor = self.advisor.clone();
                self.lessons = PendingAnswer::spawn(async move {
                    let question = Question {
                        asker: ASKER,
                        instructions: INSTRUCTIONS,
                        input: &input,
                        deadline: Instant::now() + LESSONS_DEADLINE,
                        attempt_timeout: LESSONS_ATTEMPT,
                    };
                    match advisor.review(&call, &question).await {
                        Some(lessons) if lessons.is_empty() => Lessons::Empty,
                        Some(lessons) => Lessons::Given(lessons),
                        None => Lessons::Failed,
                    }
                });
                Lessons::Asking
            }
            Err(off) => {
                self.advisor
                    .log(&format!("Advice {ASKER}: not asked, {off:?}"));
                Lessons::Off(off)
            }
        };
        self.recap = Some(recap);
        self.changed = true;
        self.window = RecapWindow::Awaiting;
    }
}
