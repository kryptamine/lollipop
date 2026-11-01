//! The `/lol-gameflow/v1/session` JSON document.

use serde::Deserialize;

use super::connection::Connection;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Session {
    game_data: GameData,
}

#[derive(Deserialize)]
struct GameData {
    queue: Queue,
}

#[derive(Deserialize)]
struct Queue {
    id: u32,
}

pub(super) async fn queue_id(connection: &Connection<'_>) -> Option<u32> {
    let session: Session = connection.get("/lol-gameflow/v1/session").await?;
    Some(session.game_data.queue.id)
}
