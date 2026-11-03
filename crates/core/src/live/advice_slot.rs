use super::arrival::Arrival;
use super::shown_advice::ShownAdvice;

const SHOWN_FOR: f64 = 20.0;

/// The one Advice the Widget holds, which every Trigger's answers compete for.
#[derive(Default)]
pub(super) struct AdviceSlot {
    showing: Option<Showing>,
    /// The texts of every Advice this game showed, oldest first.
    shown: Vec<String>,
}

struct Showing {
    arrival: Arrival,
    /// `None` until the game starts: the loading screen hides the overlay.
    from: Option<f64>,
}

impl AdviceSlot {
    /// Shows `arrival` unless a higher-ranked Advice is still showing, and says whether it did.
    /// Rank is checked here, on arrival, so a request in flight is never cancelled for it.
    pub(super) fn offer(&mut self, arrival: Arrival, game_time: Option<f64>) -> bool {
        if let Some(at) = game_time {
            self.expire(at);
        }
        let outranked = self
            .showing
            .as_ref()
            .is_some_and(|showing| showing.arrival.trigger.rank() > arrival.trigger.rank());
        if outranked {
            return false;
        }
        self.shown.push(arrival.text.clone());
        self.showing = Some(Showing {
            arrival,
            from: game_time,
        });
        true
    }

    /// An answer that came before `GameStart` counts its 20 s from `GameStart`.
    pub(super) fn at(&mut self, game_time: f64, game_start: Option<f64>) -> Option<ShownAdvice> {
        self.expire(game_time);
        let showing = self.showing.as_mut()?;
        let from = *showing
            .from
            .get_or_insert_with(|| game_start.unwrap_or(game_time));
        let left = from + SHOWN_FOR - game_time;
        Some(ShownAdvice {
            trigger: showing.arrival.trigger,
            text: showing.arrival.text.clone(),
            icon: showing.arrival.icon.clone(),
            time_left: (left / SHOWN_FOR).min(1.0),
        })
    }

    pub(super) fn shown(&self) -> &[String] {
        &self.shown
    }

    pub(super) fn clear(&mut self) {
        self.showing = None;
        self.shown.clear();
    }

    fn expire(&mut self, game_time: f64) {
        let over = self
            .showing
            .as_ref()
            .and_then(|showing| showing.from)
            .is_some_and(|from| from + SHOWN_FOR - game_time <= 0.0);
        if over {
            self.showing = None;
        }
    }
}
