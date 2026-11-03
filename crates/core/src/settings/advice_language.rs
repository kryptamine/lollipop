use serde::{Deserialize, Serialize};

use crate::advice::Language;

/// The Language setting of the Advice tab.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdviceLanguage {
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ru")]
    Russian,
}

impl AdviceLanguage {
    #[must_use]
    pub fn language(self, system_locale: Option<&str>) -> Language {
        match self {
            Self::System => Language::of_locale(system_locale),
            Self::English => Language::English,
            Self::Russian => Language::Russian,
        }
    }
}
