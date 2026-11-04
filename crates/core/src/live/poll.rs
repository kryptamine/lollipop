use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use super::clock::Clock;
use super::game_advice::GameAdvice;
use super::game_build::GameBuild;
use super::game_facts::GameFacts;
use super::game_hints::GameHints;
use super::game_markers::GameMarkers;
use super::game_notices::notices_of;
use super::gold_earned::GoldEarned;
use super::held_loading_screen::HeldLoadingScreen;
use super::inputs_of_game::InputsOfGame;
use super::league::League;
use super::live_game::LiveGame;
use super::live_update::LiveUpdate;
use super::notice::Notice;
use super::payload::{AllGameData, Player};
use super::poll_log::PollLog;
use super::snapshot::Snapshot;
use crate::advice::{Advisor, Language};
use crate::benchmarks::BenchmarkTable;
use crate::delivery::TableFile;
use crate::game_inputs::GameInputs;

pub const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Never returns.
/// # Panics
/// If the HTTP client cannot be built, which only a broken TLS backend causes.
pub async fn poll(
    league: League<'_>,
    benchmark_table: &TableFile,
    advisor: &Advisor,
    clock: &impl Clock,
    mut on_update: impl FnMut(LiveUpdate) + Send,
) {
    // League serves loopback HTTPS with a self-signed certificate, and this client
    // only ever talks to the fixed loopback URL and the lockfile's loopback port.
    #[allow(
        clippy::expect_used,
        reason = "no game data can be read without a client"
    )]
    let http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_millis(800))
        .build()
        .expect("HTTP client should build");

    let mut game = CurrentGame {
        http: &http,
        lockfile: league.lockfile,
        benchmark_table,
        gold_earned: GoldEarned::default(),
        inputs: InputsOfGame::default(),
        loading_screen: HeldLoadingScreen::default(),
        markers: GameMarkers::default(),
        hints: GameHints::default(),
        advice: GameAdvice::new(&http, league.lockfile, advisor),
        build: GameBuild::new(&http, league.lockfile, benchmark_table),
        notices: Arc::from([]),
    };
    let mut poll_log = PollLog::default();
    loop {
        let answer = fetch(&http, league.live_client_url).await;
        poll_log.answer(&answer);
        let data = answer.ok();
        let language = advisor.language();
        let update = match &data {
            Some(data) => game.update(data, language).await,
            None => None,
        };
        let between_games = data.is_none() && game.failed_poll_ends_game();
        if between_games {
            log::info!("between games");
            game.end_game();
        }
        let (build, build_change, swap) = game.build.shown(data.as_ref(), language);
        on_update(LiveUpdate {
            game: update,
            build,
            build_change,
            swap,
            between_games,
        });
        // A failed poll does not end a game, so a game still held rules out a swap too.
        if !data.is_some_and(|data| data.match_running()) && !game.inputs.holds_game() {
            benchmark_table.swap_staged();
        }
        clock.sleep(POLL_INTERVAL).await;
    }
}

/// The error says why, for the log.
async fn fetch(client: &reqwest::Client, url: &str) -> Result<AllGameData, String> {
    let response = client.get(url).send().await.map_err(|e| failure(&e))?;
    let status = response.status();
    let body = response.text().await.map_err(|e| failure(&e))?;
    serde_json::from_str(&body)
        .map_err(|e| format!("HTTP {status}, {} bytes unread: {e}", body.len()))
}

fn failure(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "timed out".to_string()
    } else if error.is_connect() {
        "connection refused".to_string()
    } else {
        error.to_string()
    }
}

struct CurrentGame<'a> {
    http: &'a reqwest::Client,
    lockfile: &'a Path,
    benchmark_table: &'a TableFile,
    gold_earned: GoldEarned,
    inputs: InputsOfGame,
    loading_screen: HeldLoadingScreen,
    markers: GameMarkers,
    hints: GameHints,
    advice: GameAdvice,
    build: GameBuild<'a>,
    notices: Arc<[Notice]>,
}

impl CurrentGame<'_> {
    fn end_held_game(&mut self) {
        if self.inputs.end_game() {
            self.benchmark_table.request_check();
        }
    }

    fn failed_poll_ends_game(&mut self) -> bool {
        !self.inputs.holds_game() && !self.loading_screen.failed_poll()
    }

    /// Also a loading screen left without a game, whose Build and answer go with it.
    fn end_game(&mut self) {
        self.advice.end_game();
        self.build.forget();
    }

    async fn update(&mut self, data: &AllGameData, language: Language) -> Option<LiveGame> {
        if data.at_loading_screen() {
            self.loading_screen.seen();
            self.build.at_loading_screen(data).await;
            self.advice
                .at_loading_screen(data, self.build.build_mut())
                .await;
        } else {
            self.loading_screen.end();
        }
        if !data.match_running() {
            if data.game_ended() {
                log::info!("game ended at gameTime {:.1}", data.game_time());
                self.end_game();
            }
            self.gold_earned.end_game();
            self.end_held_game();
            return None;
        }
        let player = data.active_player()?;
        let inputs = self.inputs_at(data, player).await;
        let snapshot = Snapshot {
            cs: player.scores.creep_score,
            level: player.level,
            gold_earned: self.gold_earned.observe(
                data.game_time(),
                data.current_gold(),
                player.item_counts(),
            ),
            scores: player.scores.player_scores(),
        };
        let (markers, benchmarks) = self.markers.at(data.game_time(), &snapshot);
        let facts = GameFacts {
            snapshot: &snapshot,
            benchmarks: &benchmarks,
            spawns: self.hints.spawns(data),
        };
        self.advice.at(data, &facts, self.build.build_mut()).await;
        Some(LiveGame {
            snapshot,
            inputs,
            markers,
            benchmarks,
            hints: self.hints.at(
                data,
                self.build
                    .build()
                    .elixir_at(player, data.current_gold(), language)
                    .as_ref(),
            ),
            advice: self.advice.shown(data.game_time(), data.game_start()),
            notices: Arc::clone(&self.notices),
            game_time: data.game_time(),
        })
    }

    async fn inputs_at(&mut self, data: &AllGameData, player: &Player) -> Arc<GameInputs> {
        let game_time = data.game_time();
        if let Some(inputs) = self.inputs.held_at(game_time) {
            return inputs;
        }
        // Still holding a game here means the clock turned back: its end went unseen.
        self.end_held_game();
        let inputs = GameInputs::read(
            self.http,
            self.lockfile,
            player.champion_name.clone(),
            player.position.as_deref(),
        )
        .await;
        let mut table = BenchmarkTable::read(self.benchmark_table.path());
        log::info!(
            "game start at gameTime {game_time:.1}: {:?} {:?}, GameStart at {:?}, Benchmark table read: {}",
            player.champion_name,
            player.position,
            data.game_start(),
            table.is_some()
        );
        self.notices = notices_of(&inputs, table.as_ref()).into();
        self.advice.start_game();
        self.build
            .at_game_start(table.as_mut(), &inputs, data, player);
        self.markers = GameMarkers::start(table, &inputs, player.cell_champion());
        self.hints = GameHints::start(&inputs);
        self.inputs.hold(game_time, inputs)
    }
}
