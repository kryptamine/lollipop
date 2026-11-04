use super::build::Build;
use super::build_prompt::{
    build_lines, elixir_lines, swap_lines, ELIXIR_INSTRUCTIONS, NEXT_ITEM_INSTRUCTIONS,
    SWAP_INSTRUCTIONS,
};
use super::payload::Player;
use crate::advice::{AnswerParts, Language, Trigger, COACHING};

/// What a Trigger's answer may change in the Build. One table says it to the model, in
/// the instructions and the input, and holds the answer to it.
#[derive(Clone, Copy)]
pub(super) struct Proposals {
    next_item: bool,
    swap: bool,
    elixir: bool,
}

impl Proposals {
    // ADR 0018 opens the Next item to the loading screen and a death alone.
    pub(super) const fn of(trigger: Trigger) -> Self {
        match trigger {
            Trigger::LoadingScreen => Self {
                next_item: true,
                swap: false,
                elixir: false,
            },
            Trigger::TurningPoint => Self {
                next_item: false,
                swap: false,
                elixir: false,
            },
            Trigger::Death => Self {
                next_item: true,
                swap: true,
                elixir: true,
            },
        }
    }

    /// Whether the Build must track its lead before the input is read.
    pub(super) const fn change_next_item(self) -> bool {
        self.next_item
    }

    pub(super) fn instructions(self, own: &str) -> String {
        let parts = [
            (true, COACHING),
            (true, own),
            (self.next_item, NEXT_ITEM_INSTRUCTIONS),
            (self.swap, SWAP_INSTRUCTIONS),
            (self.elixir, ELIXIR_INSTRUCTIONS),
        ];
        parts
            .into_iter()
            .filter_map(|(open, part)| open.then_some(part))
            .collect()
    }

    /// The input lines that list what the model may pick from, after the Trigger's own.
    pub(super) fn lines(self, build: &Build, player: &Player, language: Language) -> String {
        let mut lines = String::new();
        if self.next_item {
            build_lines(&mut lines, build, player, language);
        }
        if self.swap {
            swap_lines(&mut lines, build, player, language);
        }
        if self.elixir {
            elixir_lines(&mut lines, build, player);
        }
        lines
    }

    /// Hands the Build the parts of `parts` this Trigger may propose, and returns the text.
    pub(super) fn apply(
        self,
        parts: AnswerParts,
        player: Option<&Player>,
        game_time: Option<f64>,
        build: &mut Build,
    ) -> Option<String> {
        let AnswerParts {
            text,
            change,
            swap,
            elixir,
        } = parts;
        let Some(player) = player else {
            return text;
        };
        if let Some(change) = change.filter(|_| self.next_item) {
            build.change(change, player, game_time);
        }
        if let Some(swap) = swap.filter(|_| self.swap) {
            build.swap(swap, player);
        }
        if let Some(elixir) = elixir.filter(|_| self.elixir) {
            build.pick_elixir(elixir, player);
        }
        text
    }
}
