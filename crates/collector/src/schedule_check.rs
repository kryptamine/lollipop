//! Compares the first kill per objective and patch with the Objective schedule, which
//! catches a timer that moved earlier. A later spawn needs a threshold nobody has set.

mod early_kill;
mod first_kill;

use std::fmt;
use std::path::Path;

use lollipop_core::live::ObjectiveSchedule;

use crate::kept_match::ELDER_DRAGON;
use crate::store::Store;
use crate::table::{patch_label, TableError};

pub(crate) use early_kill::EarlyKill;
pub(crate) use first_kill::FirstKill;

/// For the owner to read: nothing here changes what a table holds.
#[derive(Debug)]
pub struct ScheduleCheck {
    compared: usize,
    early_kills: Vec<EarlyKill>,
    not_compared: Vec<String>,
}

impl ScheduleCheck {
    pub(crate) fn early_kills(&self) -> &[EarlyKill] {
        &self.early_kills
    }
}

/// # Errors
/// If the database cannot be read.
pub fn check_schedule(database: &Path) -> Result<ScheduleCheck, TableError> {
    let mut check = ScheduleCheck {
        compared: 0,
        early_kills: Vec::new(),
        not_compared: Vec::new(),
    };
    for kill in Store::open_read_only(database)?.first_kills()? {
        let spawn = ObjectiveSchedule::for_patch(kill.patch)
            .and_then(|schedule| first_spawn(schedule, &kill.objective));
        let Some((objective, spawn_s)) = spawn else {
            let patch = patch_label(kill.patch);
            check
                .not_compared
                .push(format!("{} on {patch}", kill.objective));
            continue;
        };
        check.compared += 1;
        let at_s = f64::from(kill.at_ms) / 1000.0;
        if at_s < spawn_s {
            check.early_kills.push(EarlyKill {
                objective,
                patch: kill.patch,
                at_s,
                spawn_s,
            });
        }
    }
    Ok(check)
}

fn first_spawn(schedule: &ObjectiveSchedule, objective: &str) -> Option<(&'static str, f64)> {
    match objective {
        "DRAGON" => Some(("Drake", schedule.drake_first)),
        ELDER_DRAGON => Some(("Elder", schedule.earliest_elder())),
        "HORDE" => Some(("Voidgrubs", schedule.voidgrubs_first)),
        "RIFTHERALD" => Some(("Herald", schedule.herald_first)),
        "BARON_NASHOR" => Some(("Baron", schedule.baron_first)),
        _ => None,
    }
}

impl fmt::Display for ScheduleCheck {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "schedule check: first kills compared: {}, before their spawn: {}",
            self.compared,
            self.early_kills.len()
        )?;
        for kill in &self.early_kills {
            write!(formatter, "\n{kill}")?;
        }
        // A renamed monster type lands here, and would otherwise silence its check.
        if !self.not_compared.is_empty() {
            write!(
                formatter,
                "\nnot compared: {}",
                self.not_compared.join(", ")
            )?;
        }
        Ok(())
    }
}
