use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use super::advice_slot::AdviceSlot;
use super::advised_items::AdvisedItems;
use super::clock::Clock;
use super::death_advice::DeathAdvice;
use super::game_hints::GameHints;
use super::game_markers::GameMarkers;
use super::game_notices::notices_of;
use super::gold_earned::GoldEarned;
use super::inputs_of_game::InputsOfGame;
use super::item_widget::ItemWidget;
use super::league::League;
use super::live_game::LiveGame;
use super::live_update::LiveUpdate;
use super::loading_screen_advice::LoadingScreenAdvice;
use super::notice::Notice;
use super::payload::{AllGameData, Player};
use super::snapshot::Snapshot;
use crate::advice::Advisor;
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
        advisor,
        gold_earned: GoldEarned::default(),
        inputs: InputsOfGame::default(),
        markers: GameMarkers::default(),
        hints: GameHints::default(),
        loading_screen: LoadingScreenAdvice::new(&http, league.lockfile),
        death: DeathAdvice::default(),
        items: AdvisedItems::default(),
        slot: AdviceSlot::default(),
        notices: Arc::from([]),
    };
    loop {
        let data = fetch(&http, league.live_client_url).await;
        let update = match &data {
            Some(data) => game.update(data).await,
            None => None,
        };
        let between_games = data.is_none() && !game.inputs.holds_game();
        if between_games {
            advisor.end_game();
        }
        on_update(LiveUpdate {
            game: update,
            between_games,
        });
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
    advisor: &'a Advisor,
    gold_earned: GoldEarned,
    inputs: InputsOfGame,
    markers: GameMarkers,
    hints: GameHints,
    loading_screen: LoadingScreenAdvice,
    death: DeathAdvice,
    items: AdvisedItems,
    slot: AdviceSlot,
    notices: Arc<[Notice]>,
}

impl CurrentGame<'_> {
    fn end_held_game(&mut self) {
        if self.inputs.end_game() {
            self.benchmark_table.request_check();
        }
    }

    async fn update(&mut self, data: &AllGameData) -> Option<LiveGame> {
        if data.at_loading_screen() {
            self.loading_screen
                .at_loading_screen(data, self.advisor, &mut self.slot)
                .await;
        }
        if !data.match_running() {
            if data.game_ended() {
                self.loading_screen.end_game();
                self.death.end_game();
                self.slot.clear();
                self.advisor.end_game();
            }
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
            scores: player.scores.player_scores(),
        };
        let (markers, benchmarks) = self.markers.at(data.game_time(), &snapshot);
        self.loading_screen.at(data, &mut self.slot).await;
        let mut widget = ItemWidget {
            items: &mut self.items,
            slot: &mut self.slot,
        };
        self.death.at(data, player, self.advisor, &mut widget).await;
        Some(LiveGame {
            snapshot,
            inputs,
            markers,
            benchmarks,
            hints: self.hints.at(data),
            advice: self.slot.at(data.game_time(), data.game_start()),
            notices: Arc::clone(&self.notices),
            game_time: data.game_time(),
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
        let mut table = BenchmarkTable::read(self.benchmark_table.path());
        self.notices = notices_of(&inputs, table.as_ref()).into();
        self.death.end_game();
        self.death = DeathAdvice::default();
        self.items = AdvisedItems::start(table.as_mut(), &inputs);
        self.markers = GameMarkers::start(table, &inputs, player.cell_champion());
        self.hints = GameHints::start(&inputs);
        self.inputs.hold(game_time, inputs)
    }
}
