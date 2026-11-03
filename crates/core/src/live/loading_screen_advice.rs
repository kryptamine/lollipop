use std::path::Path;

use tokio::task::JoinHandle;

use super::lane_opponent::lane_of;
use super::loading_screen_prompt::{game_lines, input, INSTRUCTIONS};
use super::payload::AllGameData;
use super::shown_advice::ShownAdvice;
use crate::advice::{Advisor, Trigger};
use crate::game_inputs::loading_tier;

const SHOWN_FOR: f64 = 20.0;
// `gameTime` after which a late answer is dropped rather than shown.
const DROP_AFTER: f64 = 90.0;

/// The lane plan asked for once per game on the loading screen.
#[derive(Default)]
pub(super) struct LoadingScreenAdvice {
    /// Since the last poll of a running game, so the next loading screen asks again.
    asked: bool,
    asking: Option<JoinHandle<Option<String>>>,
    answer: Option<Answered>,
}

struct Answered {
    text: String,
    /// `None` until the game starts: the loading screen hides the overlay.
    shown_from: Option<f64>,
}

impl LoadingScreenAdvice {
    pub(super) async fn at_loading_screen(
        &mut self,
        data: &AllGameData,
        advisor: &Advisor,
        http: &reqwest::Client,
        lockfile: &Path,
    ) {
        if self.asked {
            self.collect(None).await;
            return;
        }
        // The first answer of a loading screen can come before the player is listed.
        let Some(player) = data.active_player() else {
            return;
        };
        self.end_game();
        self.asked = true;
        let Some(call) = advisor.call_for(Trigger::LoadingScreen) else {
            return;
        };
        let Some(lane) = lane_of(data, player) else {
            return;
        };
        let game = game_lines(data, player, &lane);
        let openai = advisor.openai();
        let http = http.clone();
        let lockfile = lockfile.to_path_buf();
        self.asking = Some(tokio::spawn(async move {
            let tier = loading_tier(&http, &lockfile).await;
            let input = input(call.language, tier, &game);
            let answer = openai.ask(&call.key, INSTRUCTIONS, &input).await?;
            // No item is a candidate here, so naming one drops the whole Advice.
            let advice = answer.item.is_none().then_some(answer.advice).flatten();
            advice.filter(|advice| !advice.trim().is_empty())
        }));
    }

    pub(super) async fn at(&mut self, data: &AllGameData) -> Option<ShownAdvice> {
        self.asked = false;
        let game_time = data.game_time();
        self.collect(Some(game_time)).await;
        let answer = self.answer.as_mut()?;
        let from = *answer
            .shown_from
            .get_or_insert_with(|| data.game_start().unwrap_or(game_time));
        let left = from + SHOWN_FOR - game_time;
        if left <= 0.0 {
            self.answer = None;
            return None;
        }
        Some(ShownAdvice {
            trigger: Trigger::LoadingScreen,
            text: answer.text.clone(),
            time_left: (left / SHOWN_FOR).min(1.0),
        })
    }

    pub(super) fn end_game(&mut self) {
        if let Some(asking) = self.asking.take() {
            asking.abort();
        }
        self.answer = None;
    }

    /// Takes a finished answer. `game_time` is `None` on the loading screen.
    async fn collect(&mut self, game_time: Option<f64>) {
        let Some(asking) = self.asking.take_if(|asking| asking.is_finished()) else {
            return;
        };
        if game_time.is_some_and(|at| at >= DROP_AFTER) {
            return;
        }
        let text = asking.await.ok().flatten();
        self.answer = text.map(|text| Answered {
            text,
            shown_from: game_time,
        });
    }
}
