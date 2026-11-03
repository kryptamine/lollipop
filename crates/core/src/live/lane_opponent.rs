use super::payload::{AllGameData, Player};

const BOTTOM_LANE: [&str; 2] = ["BOTTOM", "UTILITY"];

/// The Lane opponent of the active player, and their own lane partner in the bottom lane.
pub(super) struct Lane<'a> {
    pub(super) opponents: Vec<&'a Player>,
    pub(super) partner: Option<&'a Player>,
}

/// `None` without a position (`NONE` in Practice Tool) or with no enemy in it.
pub(super) fn lane_of<'a>(data: &'a AllGameData, player: &Player) -> Option<Lane<'a>> {
    let position = player.position.as_deref().filter(|p| is_lane(p))?;
    let team = player.team()?;
    let positions: &[&str] = if BOTTOM_LANE.contains(&position) {
        &BOTTOM_LANE
    } else {
        &[position]
    };
    let in_lane = |p: &&Player| {
        p.position
            .as_deref()
            .is_some_and(|at| positions.contains(&at))
    };
    let opponents: Vec<&Player> = data
        .players()
        .iter()
        .filter(|p| p.team().is_some_and(|t| t != team))
        .filter(in_lane)
        .collect();
    if opponents.is_empty() {
        return None;
    }
    let partner = data
        .players()
        .iter()
        .filter(|p| p.team() == Some(team) && !std::ptr::eq(*p, player))
        .find(|p| positions.len() > 1 && in_lane(p));
    Some(Lane { opponents, partner })
}

fn is_lane(position: &str) -> bool {
    ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"].contains(&position)
}
