use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use super::advice_slot::AdviceSlot;
use super::build::Build;
use super::build_item::BuildItem;
use super::clock::Clock;
use super::death_advice::DeathAdvice;
use super::game_hints::GameHints;
use super::game_markers::GameMarkers;
use super::game_notices::notices_of;
use super::gold_earned::GoldEarned;
use super::inputs_of_game::InputsOfGame;
use super::league::League;
use super::live_game::LiveGame;
use super::live_update::LiveUpdate;
use super::loading_screen_advice::LoadingScreenAdvice;
use super::notice::Notice;
use super::payload::{AllGameData, Player};
use super::shown_swap::ShownSwap;
use super::snapshot::Snapshot;
use super::turning_point_advice::TurningPointAdvice;
use crate::advice::{Advisor, Language};
use crate::benchmarks::BenchmarkTable;
use crate::delivery::TableFile;
use crate::game_inputs::{loading_patch, ClientInputs, GameInputs, Role};

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
        loading_screen: LoadingScreenAdvice::new(&http, league.lockfile, advisor),
        death: DeathAdvice::new(advisor),
        turning_point: TurningPointAdvice::new(advisor),
        build: Build::default(),
        build_carried: false,
        slot: AdviceSlot::default(),
        notices: Arc::from([]),
    };
    loop {
        let data = fetch(&http, league.live_client_url).await;
        let language = advisor.language();
        let update = match &data {
            Some(data) => game.update(data, language).await,
            None => None,
        };
        let between_games = data.is_none() && !game.inputs.holds_game();
        if between_games {
            advisor.end_game();
            // A loading screen left without a game must not carry its Build into the next.
            game.forget_build();
        }
        let (build, build_change, swap) = game.build_at(data.as_ref(), language);
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
    turning_point: TurningPointAdvice,
    build: Build,
    /// The Build was read on this game's loading screen, so its start keeps it.
    build_carried: bool,
    slot: AdviceSlot,
    notices: Arc<[Notice]>,
}

impl CurrentGame<'_> {
    fn end_held_game(&mut self) {
        if self.inputs.end_game() {
            self.benchmark_table.request_check();
        }
    }

    /// Once per loading screen. Only the patch is read from the client, which may be
    /// mid-load, and the role only from the payload.
    async fn build_at_loading_screen(&mut self, data: &AllGameData) {
        if self.build_carried {
            return;
        }
        let Some(player) = data.active_player() else {
            return;
        };
        let patch = loading_patch(self.http, self.lockfile).await;
        let role = player.position.as_deref().and_then(Role::parse);
        let mut table = BenchmarkTable::read(self.benchmark_table.path());
        self.build = Build::start(table.as_mut(), patch, role, data, player);
        self.build_carried = true;
    }

    fn forget_build(&mut self) {
        self.build = Build::default();
        self.build_carried = false;
    }

    /// The Build, the reason of a Build change while it shows in a running match, and
    /// the Swap.
    fn build_at(
        &mut self,
        data: Option<&AllGameData>,
        language: Language,
    ) -> (Vec<BuildItem>, Option<String>, Option<ShownSwap>) {
        let running = data.filter(|data| data.at_loading_screen() || data.match_running());
        let (Some(data), Some(player)) = (running, running.and_then(AllGameData::active_player))
        else {
            return (Vec::new(), None, None);
        };
        let change = if data.match_running() {
            self.build.change_at(data.game_time(), data.game_start())
        } else {
            None
        };
        let swap = self.build.swap_at(player, language);
        (self.build.at(player, language), change, swap)
    }

    async fn update(&mut self, data: &AllGameData, language: Language) -> Option<LiveGame> {
        if data.at_loading_screen() {
            self.build_at_loading_screen(data).await;
            self.loading_screen
                .at_loading_screen(data, &mut self.slot, &mut self.build)
                .await;
        }
        if !data.match_running() {
            if data.game_ended() {
                self.loading_screen.end_game();
                self.death.end_game();
                self.turning_point.end_game();
                self.slot.clear();
                self.advisor.end_game();
                self.forget_build();
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
        self.loading_screen
            .at(data, &mut self.slot, &mut self.build)
            .await;
        self.turning_point
            .at(data, player, &mut self.slot, &mut self.build)
            .await;
        self.death.at(data, &mut self.slot, &mut self.build).await;
        Some(LiveGame {
            snapshot,
            inputs,
            markers,
            benchmarks,
            hints: self.hints.at(
                data,
                self.build
                    .elixir_at(player, data.current_gold(), language)
                    .as_ref(),
            ),
            advice: self.slot.at(data.game_time(), data.game_start()),
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
        self.notices = notices_of(&inputs, table.as_ref()).into();
        self.death = DeathAdvice::new(self.advisor);
        self.turning_point = TurningPointAdvice::new(self.advisor);
        // A loading screen that gave no Build, say for want of a position, gets another try.
        if !std::mem::take(&mut self.build_carried) || self.build.is_empty() {
            let patch = match &inputs.client {
                ClientInputs::Read(client) => client.patch,
                ClientInputs::Unavailable => None,
            };
            self.build = Build::start(table.as_mut(), patch, inputs.role, data, player);
        }
        self.markers = GameMarkers::start(table, &inputs, player.cell_champion());
        self.hints = GameHints::start(&inputs);
        self.inputs.hold(game_time, inputs)
    }
}
