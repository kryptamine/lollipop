use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use super::clock::Clock;
use super::game_hints::GameHints;
use super::game_markers::GameMarkers;
use super::gold_earned::GoldEarned;
use super::inputs_of_game::InputsOfGame;
use super::live_game::LiveGame;
use super::payload::{AllGameData, Player};
use super::snapshot::Snapshot;
use crate::delivery::TableFile;
use crate::game_inputs::GameInputs;

pub const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Never returns. `None` means no match the player is part of is running.
/// # Panics
/// If the HTTP client cannot be built, which only a broken TLS backend causes.
pub async fn poll(
    url: &str,
    lockfile: &Path,
    benchmark_table: &TableFile,
    clock: &impl Clock,
    mut on_update: impl FnMut(Option<LiveGame>) + Send,
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
        lockfile,
        benchmark_table,
        gold_earned: GoldEarned::default(),
        inputs: InputsOfGame::default(),
        markers: GameMarkers::default(),
        hints: GameHints::default(),
    };
    loop {
        let data = fetch(&http, url).await;
        let update = match &data {
            Some(data) => game.update(data).await,
            None => None,
        };
        on_update(update);
        // A failed poll does not end a game, so a game still held rules out a swap too.
        if !data.is_some_and(|data| data.match_running()) && !game.inputs.holds_game() {
            benchmark_table.swap_staged();
        }
        clock.sleep(POLL_INTERVAL).await;
    }
}

async fn fetch(client: &reqwest::Client, url: &str) -> Option<AllGameData> {
    let body = client.get(url).send().await.ok()?.text().await.ok()?;
    serde_json::from_str(&body).ok()
}

struct CurrentGame<'a> {
    http: &'a reqwest::Client,
    lockfile: &'a Path,
    benchmark_table: &'a TableFile,
    gold_earned: GoldEarned,
    inputs: InputsOfGame,
    markers: GameMarkers,
    hints: GameHints,
}

impl CurrentGame<'_> {
    fn end_held_game(&mut self) {
        if self.inputs.end_game() {
            self.benchmark_table.request_check();
        }
    }

    async fn update(&mut self, data: &AllGameData) -> Option<LiveGame> {
        if !data.match_running() {
            self.gold_earned.end_game();
            self.end_held_game();
            return None;
        }
        let player = data.active_player()?;
        let inputs = self.inputs_at(data.game_time(), player).await;
        let snapshot = Snapshot {
            cs: player.scores.creep_score,
            level: player.level,
            gold_earned: self.gold_earned.observe(
                data.game_time(),
                data.current_gold(),
                player.item_counts(),
            ),
        };
        let markers = self.markers.at(data.game_time(), &snapshot);
        Some(LiveGame {
            snapshot,
            inputs,
            markers,
            hints: self.hints.at(data),
        })
    }

    async fn inputs_at(&mut self, game_time: f64, player: &Player) -> Arc<GameInputs> {
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
        self.markers =
            GameMarkers::start(self.benchmark_table.path(), &inputs, player.cell_champion());
        self.hints = GameHints::start(&inputs);
        self.inputs.hold(game_time, inputs)
    }
}
