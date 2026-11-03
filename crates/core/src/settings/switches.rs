use std::collections::BTreeMap;

use serde::Serialize;

/// Every switch the settings window draws.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Switches {
    pub(super) launch_at_login: bool,
    pub(super) widgets: BTreeMap<&'static str, bool>,
}
