//! What the player chose in the menu bar menu, kept in a local file.

mod widget;

use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, io};

use serde::{Deserialize, Serialize};

pub use widget::{Widget, WIDGETS};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    // Only what the player switched is stored, so a Widget a later version adds starts on.
    widgets: BTreeMap<String, bool>,
    pub launch_at_login: bool,
}

impl Settings {
    /// A missing or unreadable file gives the defaults: settings never stop the app.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// # Errors
    /// If the file cannot be written. The previous file then stays whole.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(self)?)?;
        fs::rename(&temporary, path)
    }

    #[must_use]
    pub fn widget_on(&self, id: &str) -> bool {
        self.widgets.get(id).copied().unwrap_or(true)
    }

    pub fn set_widget(&mut self, id: &str, on: bool) {
        self.widgets.insert(id.to_string(), on);
    }

    /// Every Widget by id, the payload the view draws from.
    #[must_use]
    pub fn widget_switches(&self) -> BTreeMap<&'static str, bool> {
        WIDGETS
            .iter()
            .map(|widget| (widget.id, self.widget_on(widget.id)))
            .collect()
    }
}
