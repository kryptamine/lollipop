use std::path::Path;

use super::build::Build;
use super::build_item::BuildItem;
use super::payload::{AllGameData, Player};
use super::shown_swap::ShownSwap;
use crate::advice::Language;
use crate::benchmarks::BenchmarkTable;
use crate::delivery::TableFile;
use crate::game_inputs::{loading_patch, ClientInputs, GameInputs, Role};

/// The Build of the game being played, from its loading screen to its end.
pub(super) struct GameBuild<'a> {
    http: &'a reqwest::Client,
    lockfile: &'a Path,
    benchmark_table: &'a TableFile,
    build: Build,
    /// The Build was read on this game's loading screen, so its start keeps it.
    carried: bool,
}

impl<'a> GameBuild<'a> {
    pub(super) fn new(
        http: &'a reqwest::Client,
        lockfile: &'a Path,
        benchmark_table: &'a TableFile,
    ) -> Self {
        Self {
            http,
            lockfile,
            benchmark_table,
            build: Build::default(),
            carried: false,
        }
    }

    /// Once per loading screen. Only the patch is read from the client, which may be
    /// mid-load, and the role only from the payload.
    pub(super) async fn at_loading_screen(&mut self, data: &AllGameData) {
        if self.carried {
            return;
        }
        let Some(player) = data.active_player() else {
            return;
        };
        let patch = loading_patch(self.http, self.lockfile).await;
        let role = player.position.as_deref().and_then(Role::parse);
        let mut table = BenchmarkTable::read(self.benchmark_table.path());
        self.build = Build::start(table.as_mut(), patch, role, data, player);
        self.carried = true;
    }

    pub(super) fn at_game_start(
        &mut self,
        table: Option<&mut BenchmarkTable>,
        inputs: &GameInputs,
        data: &AllGameData,
        player: &Player,
    ) {
        // A loading screen that gave no Build, say for want of a position, gets another try.
        if std::mem::take(&mut self.carried) && !self.build.is_empty() {
            return;
        }
        let patch = match &inputs.client {
            ClientInputs::Read(client) => client.patch,
            ClientInputs::Unavailable => None,
        };
        self.build = Build::start(table, patch, inputs.role, data, player);
    }

    pub(super) fn forget(&mut self) {
        self.build = Build::default();
        self.carried = false;
    }

    /// The Build, the reason of a Build change while it shows in a running match, and
    /// the Swap.
    pub(super) fn shown(
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

    pub(super) const fn build(&self) -> &Build {
        &self.build
    }

    pub(super) const fn build_mut(&mut self) -> &mut Build {
        &mut self.build
    }
}
