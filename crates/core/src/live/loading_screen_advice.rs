use std::path::{Path, PathBuf};

use super::advice_ask::AdviceAsk;
use super::advice_slot::AdviceSlot;
use super::build::Build;
use super::lane_opponent::lane_of;
use super::loading_screen_prompt::{game_lines, input, INSTRUCTIONS};
use super::payload::AllGameData;
use super::until::Until;
use crate::advice::{Advisor, Trigger};
use crate::game_inputs::loading_tier;

// `gameTime` from which a late answer is dropped rather than shown.
const DROP_AFTER: f64 = 90.0;

/// The lane plan asked for once per game on the loading screen, which may change the
/// Next item.
pub(super) struct LoadingScreenAdvice {
    /// The League client's, which the prompt reads the player's tier from.
    http: reqwest::Client,
    lockfile: PathBuf,
    /// Since the last poll of a running game, so the next loading screen asks again.
    asked: bool,
    ask: AdviceAsk,
}

impl LoadingScreenAdvice {
    pub(super) fn new(http: &reqwest::Client, lockfile: &Path, advisor: &Advisor) -> Self {
        Self {
            http: http.clone(),
            lockfile: lockfile.to_path_buf(),
            asked: false,
            ask: AdviceAsk::new(Trigger::LoadingScreen, INSTRUCTIONS, advisor),
        }
    }

    pub(super) async fn at_loading_screen(
        &mut self,
        data: &AllGameData,
        slot: &mut AdviceSlot,
        build: &mut Build,
    ) {
        if self.asked {
            self.ask.collect(data, None, slot, build).await;
            return;
        }
        // The first answer of a loading screen can come before the player is listed.
        let Some(player) = data.active_player() else {
            return;
        };
        self.end_game();
        slot.clear();
        self.asked = true;
        let Some(lane) = lane_of(data, player) else {
            return;
        };
        let http = self.http.clone();
        let lockfile = self.lockfile.clone();
        let until = Until::GameTime(DROP_AFTER);
        self.ask.ask(data, build, until, |language, build| {
            let game = game_lines(data, player, &lane, build, language);
            async move {
                let tier = loading_tier(&http, &lockfile).await;
                input(language, tier, &game)
            }
        });
    }

    pub(super) async fn at(
        &mut self,
        data: &AllGameData,
        slot: &mut AdviceSlot,
        build: &mut Build,
    ) {
        self.asked = false;
        self.ask
            .collect(data, Some(data.game_time()), slot, build)
            .await;
    }

    pub(super) fn end_game(&mut self) {
        self.asked = false;
        self.ask.end();
    }
}
