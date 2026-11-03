use std::collections::BTreeMap;

use serde::Serialize;

use super::advice_language::AdviceLanguage;

/// Every switch the settings window draws.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Switches {
    pub(super) launch_at_login: bool,
    pub(super) widgets: BTreeMap<&'static str, bool>,
    pub(super) triggers: BTreeMap<&'static str, bool>,
    pub(super) advice_language: AdviceLanguage,
}
