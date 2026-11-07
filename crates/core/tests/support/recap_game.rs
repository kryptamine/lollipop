use std::sync::Mutex;
use std::time::{Duration, Instant};

use lollipop_core::advice::{AdviceCall, Advisor, Language, LessonsOff};
use lollipop_core::delivery::TableFile;
use lollipop_core::live::{poll, GameHistory, League, Lessons, LiveUpdate, OnDisk, Recap};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::sync::mpsc;

use super::benchmark_file::{cell, minute, table_of, ENOUGH_GAMES};
use super::fake_lcu::FakeLcu;
use super::game_clock::GameClock;
use super::loading_game::{lineup, KEY, ME};
use super::server::serve;

pub const PHASE: &str = "/lol-gameflow/v1/gameflow-phase";
pub const KILLER: &str = "Foe2#EUW";
// Polls the loop makes past the script, all failing, before a run counts as settled.
const POLLS_PAST_SCRIPT: usize = 5;

/// One poll of a scripted game.
#[derive(Clone, Copy)]
pub enum Beat {
    Running {
        at: f64,
        cs: u32,
    },
    /// The poll that sees `GameEnd`, with its `Result`.
    Ended {
        at: f64,
        cs: u32,
        result: &'static str,
    },
    /// Live Client Data answers nothing, with the League client in `phase`.
    Gone(&'static str),
    /// Neither Live Client Data nor the League client's gameflow phase answers.
    Unanswered,
}

/// A running game every 30 s from 0:00 to `minutes`, the player farming `cs_per_minute`.
pub fn farming(minutes: u32, cs_per_minute: u32) -> Vec<Beat> {
    (0..=minutes * 2)
        .map(|half| Beat::Running {
            at: f64::from(half) * 30.0,
            cs: cs_per_minute * half / 2,
        })
        .collect()
}

/// Ahri's Cell in the middle lane at Gold: 8 CS and 400 gold a minute over 500.
pub fn ahri_table() -> String {
    let minutes: Vec<Value> = (0..=40)
        .map(|at| {
            let at_f = f64::from(at);
            minute(
                at,
                8.0 * at_f,
                400.0f64.mul_add(at_f, 500.0),
                10.0,
                ENOUGH_GAMES,
            )
        })
        .collect();
    table_of(&[cell("Ahri", "MIDDLE", "GOLD", &minutes)])
}

/// What the poll loop sent of the Recap over a whole scripted game.
pub struct Played {
    /// Every Recap sent, in order.
    pub recaps: Vec<Recap>,
    /// The polls, counted from 0, at which the Recap window was to open.
    pub opened_at: Vec<usize>,
}

impl Played {
    pub fn last(&self) -> &Recap {
        self.recaps.last().unwrap()
    }
}

/// Ahri in the middle lane against Zed, the League client, a table and a history on
/// disk. Games played on one bench share its history.
pub struct RecapBench {
    lcu: FakeLcu,
    dir: TempDir,
    /// Game times the player dies to Zed at.
    pub deaths: Vec<f64>,
    pub champion: &'static str,
    pub history: std::path::PathBuf,
}

impl RecapBench {
    pub async fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let bench = Self {
            lcu: FakeLcu::start().await,
            history: dir.path().join("history.sqlite"),
            dir,
            deaths: Vec::new(),
            champion: "Ahri",
        };
        bench.lcu.set(PHASE, r#""InProgress""#);
        std::fs::write(bench.table(), ahri_table()).unwrap();
        bench
    }

    fn table(&self) -> std::path::PathBuf {
        self.dir.path().join("benchmark-table.json")
    }

    /// Plays `beats` with no OpenAI key.
    pub async fn play(&self, beats: Vec<Beat>) -> Played {
        self.play_asking(beats, "http://127.0.0.1:9", Err(LessonsOff::NoKey))
            .await
    }

    /// Plays `beats` with every Trigger off, asking OpenAI at `openai_url` for Lessons
    /// unless `lessons` says why not, and waits for them.
    pub async fn play_asking(
        &self,
        beats: Vec<Beat>,
        openai_url: &str,
        lessons: Result<(), LessonsOff>,
    ) -> Played {
        let advisor = Advisor::new(openai_url, |_| None).reviewing(move || {
            lessons.map(|()| AdviceCall {
                key: KEY.to_string(),
                language: Language::Russian,
            })
        });
        let script = beats.len();
        let mut updates = self.spawn_poll(beats, advisor).await;
        let mut played = Played {
            recaps: Vec::new(),
            opened_at: Vec::new(),
        };
        let mut polls = 0;
        // Past the Lessons' own 30 s deadline, a run that never settles fails, not hangs.
        let give_up = Instant::now() + Duration::from_secs(45);
        let settled = |played: &Played, polls: usize| {
            polls >= script + POLLS_PAST_SCRIPT
                && played
                    .recaps
                    .last()
                    .is_none_or(|r| r.lessons != Lessons::Asking)
        };
        while !settled(&played, polls) {
            assert!(Instant::now() < give_up, "the Recap never settled");
            let update = tokio::time::timeout(Duration::from_secs(10), updates.recv())
                .await
                .unwrap()
                .unwrap();
            played.recaps.extend(update.recap.recap);
            if update.recap.open {
                played.opened_at.push(polls);
            }
            polls += 1;
            // Past the script every poll fails at once, so give OpenAI time to answer.
            if polls > script {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        }
        played
    }

    async fn spawn_poll(
        &self,
        beats: Vec<Beat>,
        advisor: Advisor,
    ) -> mpsc::UnboundedReceiver<LiveUpdate> {
        let served = Mutex::new(beats.into_iter());
        let (route, deaths, champion) = (self.lcu.router(), self.deaths.clone(), self.champion);
        let url = serve(Box::new(move || {
            let beat = served.lock().unwrap().next()?;
            match beat {
                Beat::Running { at, cs } => Some(body(champion, at, cs, &deaths, None)),
                Beat::Ended { at, cs, result } => {
                    Some(body(champion, at, cs, &deaths, Some(result)))
                }
                Beat::Gone(phase) => {
                    route(PHASE, Some(&format!("\"{phase}\"")));
                    None
                }
                Beat::Unanswered => {
                    route(PHASE, None);
                    None
                }
            }
        }))
        .await;
        let (updates, received) = mpsc::unbounded_channel();
        let (lockfile, table) = (self.lcu.lockfile(), self.table());
        let history = GameHistory::new(self.history.clone());
        tokio::spawn(async move {
            poll(
                League {
                    live_client_url: &url,
                    lockfile: &lockfile,
                },
                OnDisk {
                    benchmark_table: &TableFile::new(table),
                    history: &history,
                },
                &advisor,
                &GameClock::starting_at(0.0),
                move |update| {
                    let _ = updates.send(update);
                },
            )
            .await;
        });
        received
    }
}

fn body(champion: &str, at: f64, cs: u32, deaths: &[f64], result: Option<&str>) -> String {
    let mut events = vec![json!({ "EventName": "GameStart", "EventTime": 0.0 })];
    for death in deaths.iter().filter(|death| **death <= at) {
        events.push(json!({
            "EventName": "ChampionKill",
            "EventTime": death,
            "KillerName": KILLER,
            "VictimName": ME,
            "Assisters": [],
        }));
    }
    if let Some(result) = result {
        events.push(json!({ "EventName": "GameEnd", "EventTime": at, "Result": result }));
    }
    let mut players = lineup(champion, "MIDDLE");
    for player in &mut players {
        player["scores"] = json!({ "creepScore": 0, "kills": 1, "deaths": 2, "assists": 3 });
        if player["riotId"] == ME {
            player["scores"]["creepScore"] = json!(cs);
            player["rawChampionName"] = json!(format!("game_character_displayname_{champion}"));
        }
    }
    // Gold earned grows 300 a minute over the starting 500, with nothing bought.
    let gold = 300.0f64.mul_add(at / 60.0, 500.0);
    json!({
        "activePlayer": { "riotId": ME, "currentGold": gold },
        "allPlayers": players,
        "events": { "Events": events },
        "gameData": { "gameTime": at },
    })
    .to_string()
}

/// A Lessons reply as the model writes it.
pub fn lessons_reply(lessons: &[&str]) -> String {
    json!({ "lessons": lessons }).to_string()
}
