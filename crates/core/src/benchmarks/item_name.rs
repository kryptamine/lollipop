use serde::Deserialize;

use crate::advice::Language;

/// An item's name from Data Dragon, in each language Advice is written in.
#[derive(Debug, Deserialize)]
pub(crate) struct ItemName {
    en: String,
    ru: String,
}

impl ItemName {
    pub(crate) fn of(&self, language: Language) -> &str {
        match language {
            Language::English => &self.en,
            Language::Russian => &self.ru,
        }
    }
}
