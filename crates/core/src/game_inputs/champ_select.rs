//! The `/lol-champ-select/v1/session` JSON document.

use serde::Deserialize;

use super::connection::Connection;
use super::role::Role;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Session {
    local_player_cell_id: i64,
    my_team: Vec<Member>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Member {
    cell_id: i64,
    assigned_position: String,
}

pub(super) async fn assigned_role(connection: &Connection<'_>) -> Option<Role> {
    let session: Session = connection.get("/lol-champ-select/v1/session").await?;
    let me = session
        .my_team
        .iter()
        .find(|member| member.cell_id == session.local_player_cell_id)?;
    Role::parse(&me.assigned_position)
}
