use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use super::clock::Clock;
use super::game_markers::GameMarkers;
use super::gold_earned::GoldEarned;
use super::inputs_of_game::InputsOfGame;
use super::live_game::LiveGame;
use super::payload::{AllGameData, Player};
use super::snapshot::Snapshot;
use crate::game_inputs::GameInputs;

pub const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Never returns. `None` means no match the player is part of is running.
/// # Panics
/// If the HTTP client cannot be built, which only a broken TLS backend causes.
pub async fn poll(
    url: &str,
    lockfile: &Path,
    benchmark_table: &Path,
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
    };
    loop {
        let update = match fetch(&http, url).await {
            Some(data) => game.update(&data).await,
            None => None,
        };
        on_update(update);
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
    benchmark_table: &'a Path,
    gold_earned: GoldEarned,
    inputs: InputsOfGame,
    markers: GameMarkers,
}

impl CurrentGame<'_> {
    async fn update(&mut self, data: &AllGameData) -> Option<LiveGame> {
        if !data.match_running() {
            self.gold_earned.end_game();
            self.inputs.end_game();
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
        })
    }

    async fn inputs_at(&mut self, game_time: f64, player: &Player) -> Arc<GameInputs> {
        if let Some(inputs) = self.inputs.held_at(game_time) {
            return inputs;
        }
        let inputs = GameInputs::read(
            self.http,
            self.lockfile,
            player.champion_name.clone(),
            player.position.as_deref(),
        )
        .await;
        self.markers = GameMarkers::start(self.benchmark_table, &inputs, player.cell_champion());
        self.inputs.hold(game_time, inputs)
    }
}
