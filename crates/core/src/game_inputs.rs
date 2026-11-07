//! What Markers and Timing hints depend on, read once when a game starts.

mod champ_select;
mod client_inputs;
mod client_phase;
mod client_reading;
mod connection;
mod gameflow_session;
mod loading_patch;
mod loading_tier;
mod lockfile;
mod patch;
mod ranked_stats;
mod role;
mod tier;
mod tier_reading;

use std::path::Path;

use connection::Connection;
use lockfile::Lockfile;

pub use client_inputs::ClientInputs;
pub(crate) use client_phase::ClientPhase;
pub use client_reading::ClientReading;
pub(crate) use loading_patch::loading_patch;
pub(crate) use loading_tier::loading_tier;
pub use lockfile::LOCKFILE_PATH;
pub use patch::Patch;
pub use role::Role;
pub use tier::Tier;
pub use tier_reading::TierReading;

#[derive(Debug, PartialEq, Eq)]
pub struct GameInputs {
    pub champion: Option<String>,
    pub role: Option<Role>,
    pub client: ClientInputs,
}

impl GameInputs {
    pub(crate) async fn read(
        http: &reqwest::Client,
        lockfile: &Path,
        champion: Option<String>,
        position: Option<&str>,
    ) -> Self {
        let live_role = position.and_then(Role::parse);
        let Some(lockfile) = Lockfile::read(lockfile) else {
            return Self {
                champion,
                role: live_role,
                client: ClientInputs::Unavailable,
            };
        };
        let connection = Connection::new(http, lockfile);
        let (role, client) = tokio::join!(
            role_of(&connection, live_role),
            ClientInputs::read(&connection),
        );
        Self {
            champion,
            role,
            client,
        }
    }
}

impl GameInputs {
    /// The solo/duo tier, `None` when the client was not read or the player is unranked.
    #[must_use]
    pub const fn ranked_tier(&self) -> Option<Tier> {
        match &self.client {
            ClientInputs::Read(ClientReading {
                tier: TierReading::Ranked(tier),
                ..
            }) => Some(*tier),
            _ => None,
        }
    }
}

async fn role_of(connection: &Connection<'_>, live_role: Option<Role>) -> Option<Role> {
    match live_role {
        Some(role) => Some(role),
        None => champ_select::assigned_role(connection).await,
    }
}
