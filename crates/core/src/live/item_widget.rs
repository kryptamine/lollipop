use super::advice_slot::AdviceSlot;
use super::advised_items::AdvisedItems;
use super::answered::Answered;
use super::arrival::Arrival;
use super::payload::Player;
use crate::advice::Trigger;

/// The Advice Widget as a Trigger that may name an item reaches it, with the items it
/// showed this game.
pub(super) struct ItemWidget<'a> {
    pub(super) items: &'a mut AdvisedItems,
    pub(super) slot: &'a mut AdviceSlot,
}

impl ItemWidget<'_> {
    /// An item counts as recommended only once its Advice shows.
    pub(super) fn offer(
        &mut self,
        answered: Answered,
        trigger: Trigger,
        game_time: f64,
        player: &Player,
    ) {
        let arrival = Arrival {
            trigger,
            text: answered.text,
            icon: answered.item.and_then(|item| self.items.icon_url(item)),
        };
        if self.slot.offer(arrival, Some(game_time)) {
            if let Some(item) = answered.item {
                self.items.show(item, player);
            }
        }
    }
}
