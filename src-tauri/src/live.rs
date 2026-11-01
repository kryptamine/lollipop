//! Parses Riot's Live Client Data API and derives the numbers the overlay shows.
//! Only the active player's own stats are read.

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AllGameData {
    active_player: ActivePlayer,
    all_players: Vec<Player>,
    game_data: GameData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivePlayer {
    riot_id: Option<String>,
    summoner_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Player {
    riot_id: Option<String>,
    summoner_name: Option<String>,
    champion_name: String,
    scores: Scores,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scores {
    creep_score: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameData {
    game_time: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Snapshot {
    pub champion: String,
    pub cs: u32,
    pub game_time_secs: f64,
    pub cs_per_min: f64,
    /// CS ahead of (+) or behind (-) the target pace.
    pub delta_to_target: i32,
}

/// Returns `None` when the payload isn't a running game the player is part of.
pub fn parse_snapshot(body: &str, target_per_min: f64) -> Option<Snapshot> {
    let data: AllGameData = serde_json::from_str(body).ok()?;
    let me = find_active_player(&data)?;

    let cs = me.scores.creep_score;
    let minutes = data.game_data.game_time.max(0.0) / 60.0;
    let cs_per_min = if minutes >= 1.0 { cs as f64 / minutes } else { 0.0 };

    Some(Snapshot {
        champion: me.champion_name.clone(),
        cs,
        game_time_secs: data.game_data.game_time,
        cs_per_min,
        delta_to_target: (cs as f64 - target_per_min * minutes).round() as i32,
    })
}

fn find_active_player(data: &AllGameData) -> Option<&Player> {
    let active = &data.active_player;
    data.all_players.iter().find(|p| {
        let same_riot_id = active.riot_id.is_some() && p.riot_id == active.riot_id;
        let same_name = active.summoner_name.is_some() && p.summoner_name == active.summoner_name;
        same_riot_id || same_name
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(game_time: f64) -> String {
        format!(
            r#"{{
              "activePlayer": {{ "riotId": "Me#EUW" }},
              "allPlayers": [
                {{ "riotId": "Enemy#EUW", "championName": "Zed", "scores": {{ "creepScore": 200 }} }},
                {{ "riotId": "Me#EUW", "championName": "Ahri", "scores": {{ "creepScore": 60 }} }}
              ],
              "gameData": {{ "gameTime": {game_time}, "gameMode": "CLASSIC" }}
            }}"#
        )
    }

    #[test]
    fn reads_only_the_active_players_cs() {
        let s = parse_snapshot(&body(600.0), 7.0).unwrap();
        assert_eq!(s.champion, "Ahri");
        assert_eq!(s.cs, 60);
    }

    #[test]
    fn computes_pace_and_delta() {
        let s = parse_snapshot(&body(600.0), 7.0).unwrap();
        assert_eq!(s.cs_per_min, 6.0);
        assert_eq!(s.delta_to_target, -10);
    }

    #[test]
    fn no_rate_spike_in_first_minute() {
        assert_eq!(parse_snapshot(&body(30.0), 7.0).unwrap().cs_per_min, 0.0);
    }

    #[test]
    fn falls_back_to_summoner_name() {
        let body = r#"{
          "activePlayer": { "summonerName": "Me" },
          "allPlayers": [{ "summonerName": "Me", "championName": "Ahri", "scores": { "creepScore": 5 } }],
          "gameData": { "gameTime": 120.0 }
        }"#;
        assert_eq!(parse_snapshot(body, 7.0).unwrap().cs, 5);
    }

    #[test]
    fn missing_ids_never_match_each_other() {
        let body = r#"{
          "activePlayer": {},
          "allPlayers": [{ "championName": "Zed", "scores": { "creepScore": 5 } }],
          "gameData": { "gameTime": 120.0 }
        }"#;
        assert_eq!(parse_snapshot(body, 7.0), None);
    }

    #[test]
    fn non_game_payload_is_none() {
        assert_eq!(parse_snapshot(r#"{"errorCode":"RESOURCE_NOT_FOUND"}"#, 7.0), None);
        assert_eq!(parse_snapshot("not json", 7.0), None);
    }
}
