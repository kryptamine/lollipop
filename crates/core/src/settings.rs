//! What the player chose in the settings window, kept in a local file.

mod advice_language;
mod switches;
mod widget;

use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::advice::{AdviceCall, Trigger, TRIGGERS};

pub use advice_language::AdviceLanguage;
pub use switches::Switches;
pub use widget::{Widget, ADVICE_WIDGET, WIDGETS};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    // Only what the player switched is stored, so a Widget or Trigger a later version
    // adds starts on.
    widgets: BTreeMap<String, bool>,
    triggers: BTreeMap<String, bool>,
    pub launch_at_login: bool,
    pub advice_language: AdviceLanguage,
    pub voiceover: bool,
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

    /// An id no Widget has is ignored, so the view cannot write junk into the file.
    pub fn set_widget(&mut self, id: &str, on: bool) {
        if WIDGETS.iter().any(|widget| widget.id == id) {
            self.widgets.insert(id.to_string(), on);
        }
    }

    #[must_use]
    pub fn trigger_on(&self, trigger: Trigger) -> bool {
        self.triggers.get(trigger.id()).copied().unwrap_or(true)
    }

    /// An id no Trigger has is ignored, as for Widgets.
    pub fn set_trigger(&mut self, id: &str, on: bool) {
        if TRIGGERS.iter().any(|trigger| trigger.id() == id) {
            self.triggers.insert(id.to_string(), on);
        }
    }

    /// `None` keeps OpenAI uncalled: no key, the Advice Widget off, or the Trigger off.
    #[must_use]
    pub fn advice_call(
        &self,
        trigger: Trigger,
        key: Option<String>,
        system_locale: Option<&str>,
    ) -> Option<AdviceCall> {
        if !self.widget_on(ADVICE_WIDGET) || !self.trigger_on(trigger) {
            return None;
        }
        Some(AdviceCall {
            key: key?,
            language: self.advice_language.language(system_locale),
        })
    }

    /// Voiceover reads only what the Advice Widget shows, so it needs the Widget on.
    #[must_use]
    pub fn reads_aloud(&self) -> bool {
        self.voiceover && self.widget_on(ADVICE_WIDGET)
    }

    /// Every Widget by id, the payload the view draws from.
    #[must_use]
    pub fn widget_switches(&self) -> BTreeMap<&'static str, bool> {
        WIDGETS
            .iter()
            .map(|widget| (widget.id, self.widget_on(widget.id)))
            .collect()
    }

    #[must_use]
    pub fn switches(&self) -> Switches {
        Switches {
            launch_at_login: self.launch_at_login,
            widgets: self.widget_switches(),
            triggers: TRIGGERS
                .iter()
                .map(|&trigger| (trigger.id(), self.trigger_on(trigger)))
                .collect(),
            advice_language: self.advice_language,
            voiceover: self.voiceover,
        }
    }
}
