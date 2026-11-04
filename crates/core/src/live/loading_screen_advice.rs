use std::path::{Path, PathBuf};

use super::build::Build;
use super::fired::{AskInput, Fired, InputFuture};
use super::lane_opponent::lane_of;
use super::loading_screen_prompt::{game_lines, input};
use super::payload::AllGameData;
use super::until::Until;
use crate::advice::Language;
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
}

impl LoadingScreenAdvice {
    pub(super) fn new(http: &reqwest::Client, lockfile: &Path) -> Self {
        Self {
            http: http.clone(),
            lockfile: lockfile.to_path_buf(),
            asked: false,
        }
    }

    /// Fires once per loading screen, which starts a game's Advice afresh.
    pub(super) fn at_loading_screen<'a>(
        &mut self,
        data: &'a AllGameData,
    ) -> Option<Fired<impl AskInput + 'a>> {
        if self.asked {
            return None;
        }
        // The first answer of a loading screen can come before the player is listed.
        let player = data.active_player()?;
        self.asked = true;
        let Some(lane) = lane_of(data, player) else {
            return Some(Fired::Skip("no lane opponent, not asked"));
        };
        let http = self.http.clone();
        let lockfile = self.lockfile.clone();
        Some(Fired::Ask {
            until: Until::GameTime(DROP_AFTER),
            input: move |language: Language, build: &Build| -> InputFuture {
                let game = game_lines(data, player, &lane, build);
                Box::pin(async move {
                    let tier = loading_tier(&http, &lockfile).await;
                    log::info!("loading screen tier: {tier:?}");
                    input(language, tier, &game)
                })
            },
        })
    }

    /// For a running game or one ended: the next loading screen asks again.
    pub(super) const fn rearm(&mut self) {
        self.asked = false;
    }
}
