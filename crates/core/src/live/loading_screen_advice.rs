use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tokio::task::JoinHandle;

use super::advice_slot::AdviceSlot;
use super::arrival::Arrival;
use super::lane_opponent::lane_of;
use super::loading_screen_prompt::{game_lines, input, INSTRUCTIONS};
use super::payload::AllGameData;
use crate::advice::{Advisor, Question, Trigger};
use crate::game_inputs::loading_tier;

// `gameTime` after which a late answer is dropped rather than shown.
const DROP_AFTER: f64 = 90.0;

/// The lane plan asked for once per game on the loading screen.
pub(super) struct LoadingScreenAdvice {
    /// The League client's, which the prompt reads the player's tier from.
    http: reqwest::Client,
    lockfile: PathBuf,
    /// Since the last poll of a running game, so the next loading screen asks again.
    asked: bool,
    asking: Option<JoinHandle<Option<String>>>,
}

impl LoadingScreenAdvice {
    pub(super) fn new(http: &reqwest::Client, lockfile: &Path) -> Self {
        Self {
            http: http.clone(),
            lockfile: lockfile.to_path_buf(),
            asked: false,
            asking: None,
        }
    }

    pub(super) async fn at_loading_screen(
        &mut self,
        data: &AllGameData,
        advisor: &Advisor,
        slot: &mut AdviceSlot,
    ) {
        if self.asked {
            self.collect(None, slot).await;
            return;
        }
        // The first answer of a loading screen can come before the player is listed.
        let Some(player) = data.active_player() else {
            return;
        };
        self.end_game();
        slot.clear();
        self.asked = true;
        let Some(call) = advisor.call_for(Trigger::LoadingScreen) else {
            return;
        };
        let Some(lane) = lane_of(data, player) else {
            return;
        };
        let game = game_lines(data, player, &lane);
        // `gameTime` starts only after the loading screen, so the deadline is further off.
        let deadline = Instant::now() + Duration::from_secs_f64(DROP_AFTER);
        let advisor = advisor.clone();
        let http = self.http.clone();
        let lockfile = self.lockfile.clone();
        self.asking = Some(tokio::spawn(async move {
            let tier = loading_tier(&http, &lockfile).await;
            let input = input(call.language, tier, &game);
            let question = Question {
                trigger: Trigger::LoadingScreen,
                instructions: INSTRUCTIONS,
                input: &input,
                deadline,
            };
            advisor.ask(&call, &question).await.into_text()
        }));
    }

    pub(super) async fn at(&mut self, data: &AllGameData, slot: &mut AdviceSlot) {
        self.asked = false;
        self.collect(Some(data.game_time()), slot).await;
    }

    pub(super) fn end_game(&mut self) {
        if let Some(asking) = self.asking.take() {
            asking.abort();
        }
    }

    /// Offers a finished answer. `game_time` is `None` on the loading screen.
    async fn collect(&mut self, game_time: Option<f64>, slot: &mut AdviceSlot) {
        let Some(asking) = self.asking.take_if(|asking| asking.is_finished()) else {
            return;
        };
        if game_time.is_some_and(|at| at >= DROP_AFTER) {
            return;
        }
        if let Some(text) = asking.await.ok().flatten() {
            let arrival = Arrival {
                trigger: Trigger::LoadingScreen,
                text,
            };
            slot.offer(arrival, game_time);
        }
    }
}
