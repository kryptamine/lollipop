//! The `allgamedata` JSON document, one struct per nested object.

use serde::Deserialize;

use super::event_name::EventName;
use super::gold_earned::ItemCounts;
use super::team::Team;

const RAW_CHAMPION_PREFIX: &str = "game_character_displayname_";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AllGameData {
    active_player: ActivePlayer,
    all_players: Vec<Player>,
    events: Events,
    game_data: GameData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivePlayer {
    riot_id: Option<String>,
    summoner_name: Option<String>,
    current_gold: f64,
    full_runes: Option<FullRunes>,
    champion_stats: Option<ChampionStats>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ChampionStats {
    current_health: Option<f64>,
    pub(super) max_health: Option<f64>,
    pub(super) armor: Option<f64>,
    pub(super) magic_resist: Option<f64>,
}

// Riot documents none of these as always there, and a field that fails to read would
// lose the whole poll, so each one is optional.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FullRunes {
    pub(super) keystone: Option<Named>,
    pub(super) primary_rune_tree: Option<Named>,
    pub(super) secondary_rune_tree: Option<Named>,
    #[serde(default)]
    pub(super) general_runes: Vec<Named>,
    #[serde(default)]
    pub(super) stat_runes: Vec<StatRune>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Runes {
    pub(super) keystone: Option<Named>,
    pub(super) primary_rune_tree: Option<Named>,
    pub(super) secondary_rune_tree: Option<Named>,
}

/// A rune, rune tree or summoner spell, by its name in the client's language.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Named {
    pub(super) display_name: Option<String>,
    /// Such as `GeneratedTip_SummonerSpell_SummonerSmite_DisplayName`, in every language.
    pub(super) raw_display_name: Option<String>,
}

/// A stat shard has no display name, only one such as `perk_tooltip_StatModAdaptive`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StatRune {
    pub(super) raw_description: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SummonerSpells {
    pub(super) summoner_spell_one: Option<Named>,
    pub(super) summoner_spell_two: Option<Named>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Player {
    riot_id: Option<String>,
    riot_id_game_name: Option<String>,
    summoner_name: Option<String>,
    #[serde(default, deserialize_with = "Team::deserialize_or_none")]
    team: Option<Team>,
    pub(super) champion_name: Option<String>,
    raw_champion_name: Option<String>,
    pub(super) position: Option<String>,
    pub(super) level: u32,
    #[serde(default)]
    pub(super) is_dead: bool,
    /// Seconds left until the player respawns, while dead.
    #[serde(default)]
    pub(super) respawn_timer: f64,
    pub(super) items: Vec<Item>,
    pub(super) scores: Scores,
    pub(super) runes: Option<Runes>,
    pub(super) summoner_spells: Option<SummonerSpells>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Item {
    #[serde(rename = "itemID")]
    pub(super) id: u32,
    count: u32,
    /// In the client's language.
    pub(super) display_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Scores {
    pub(super) creep_score: u32,
    pub(super) kills: Option<u32>,
    pub(super) deaths: Option<u32>,
    pub(super) assists: Option<u32>,
}

impl Scores {
    pub(super) fn player_scores(&self) -> Option<super::scores::Scores> {
        Some(super::scores::Scores {
            kills: self.kills?,
            deaths: self.deaths?,
            assists: self.assists?,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Events {
    events: Vec<Event>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Event {
    #[serde(rename = "EventName")]
    name: EventName,
    #[serde(rename = "EventTime")]
    time: Option<f64>,
    pub(super) killer_name: Option<String>,
    pub(super) victim_name: Option<String>,
    #[serde(default)]
    pub(super) assisters: Vec<String>,
    dragon_type: Option<String>,
    turret_killed: Option<String>,
    inhib_killed: Option<String>,
    #[serde(default, deserialize_with = "Team::deserialize_or_none")]
    acing_team: Option<Team>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameData {
    game_time: f64,
}

impl AllGameData {
    pub(super) fn active_player(&self) -> Option<&Player> {
        let active = &self.active_player;
        self.all_players.iter().find(|p| {
            let same_riot_id = active.riot_id.is_some() && p.riot_id == active.riot_id;
            let same_name =
                active.summoner_name.is_some() && p.summoner_name == active.summoner_name;
            same_riot_id || same_name
        })
    }

    pub(super) fn players(&self) -> &[Player] {
        &self.all_players
    }

    pub(super) fn enemies_of<'a>(&'a self, player: &'a Player) -> impl Iterator<Item = &'a Player> {
        self.all_players
            .iter()
            .filter(|other| other.team().is_some() && other.team() != player.team())
    }

    /// The player's team, the player included.
    pub(super) fn allies_of<'a>(&'a self, player: &'a Player) -> impl Iterator<Item = &'a Player> {
        self.all_players
            .iter()
            .filter(|other| other.team().is_some() && other.team() == player.team())
    }

    pub(super) const fn full_runes(&self) -> Option<&FullRunes> {
        self.active_player.full_runes.as_ref()
    }

    pub(super) const fn champion_stats(&self) -> Option<&ChampionStats> {
        self.active_player.champion_stats.as_ref()
    }

    pub(super) fn health_share(&self) -> Option<f64> {
        let stats = self.champion_stats()?;
        let max = stats.max_health.filter(|max| *max > 0.0)?;
        Some(stats.current_health? / max)
    }

    pub(super) const fn current_gold(&self) -> f64 {
        self.active_player.current_gold
    }

    pub(super) const fn game_time(&self) -> f64 {
        self.game_data.game_time
    }

    /// A 200 alone proves nothing: it can come about 56 s before `GameStart`.
    pub(super) fn match_running(&self) -> bool {
        self.has_event(EventName::GameStart) && !self.has_event(EventName::GameEnd)
    }

    /// League answers for about 56 s of the loading screen before it logs `GameStart`.
    pub(super) fn at_loading_screen(&self) -> bool {
        !self.has_event(EventName::GameStart)
    }

    pub(super) fn game_ended(&self) -> bool {
        self.has_event(EventName::GameEnd)
    }

    pub(super) fn game_start(&self) -> Option<f64> {
        self.events_named(EventName::GameStart)
            .next()
            .and_then(Event::time)
    }

    fn has_event(&self, name: EventName) -> bool {
        self.events.events.iter().any(|e| e.name == name)
    }

    /// The whole game's events, oldest first.
    pub(super) fn events(&self) -> &[Event] {
        &self.events.events
    }

    /// The whole game's events of one kind, oldest first.
    pub(super) fn events_named(&self, name: EventName) -> impl Iterator<Item = &Event> {
        self.events.events.iter().filter(move |e| e.name == name)
    }

    pub(super) fn killer_team(&self, kill: &Event) -> Option<Team> {
        let killer = kill.killer_name.as_deref()?;
        if !self.all_players.iter().any(|p| p.goes_by(killer)) {
            return Team::of_minion(killer);
        }
        self.player_named(killer)?.team
    }

    /// The team that took an objective or structure, or aced the other.
    pub(super) fn taker_team(&self, event: &Event) -> Option<Team> {
        if let Some(structure) = event.turret_killed.as_ref().or(event.inhib_killed.as_ref()) {
            return Team::destroyer_of(structure);
        }
        event.acing_team.or_else(|| self.killer_team(event))
    }

    /// `None` also for a name two players go by: a game name is not unique, and picking
    /// one of them could miscount the Elder rule or blame the wrong killer.
    pub(super) fn player_named(&self, name: &str) -> Option<&Player> {
        let mut named = self.all_players.iter().filter(|p| p.goes_by(name));
        let player = named.next()?;
        named.next().is_none().then_some(player)
    }
}

impl Item {
    /// The client's name for the item, or its id when the client gives none.
    pub(super) fn shown_name(&self) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| format!("item {}", self.id))
    }
}

impl Event {
    pub(super) const fn name(&self) -> EventName {
        self.name
    }

    /// `EventTime`, not the poll that saw it: an event can be logged seconds late.
    pub(super) const fn time(&self) -> Option<f64> {
        self.time
    }

    pub(super) fn is_elder(&self) -> bool {
        self.dragon_type.as_deref() == Some("Elder")
    }

    pub(super) fn dragon_type(&self) -> Option<&str> {
        self.dragon_type.as_deref()
    }
}

impl Player {
    /// The name that tells players apart from one poll to the next.
    pub(super) fn identity(&self) -> Option<&str> {
        [&self.riot_id, &self.summoner_name, &self.riot_id_game_name]
            .into_iter()
            .find_map(Option::as_deref)
    }

    /// Whether a summoner spell's raw name holds `spell`, such as `SummonerSmite`.
    pub(super) fn has_summoner(&self, spell: &str) -> bool {
        self.summoner_spells
            .iter()
            .flat_map(|spells| [&spells.summoner_spell_one, &spells.summoner_spell_two])
            .flatten()
            .filter_map(|named| named.raw_display_name.as_deref())
            .any(|raw| raw.contains(spell))
    }

    pub(super) const fn team(&self) -> Option<Team> {
        self.team
    }

    // Events give "a player name", and no recording shows which of these it is.
    pub(super) fn goes_by(&self, name: &str) -> bool {
        [&self.riot_id, &self.riot_id_game_name, &self.summoner_name]
            .into_iter()
            .any(|known| known.as_deref() == Some(name))
    }

    /// The name a Benchmark Cell goes by. `rawChampionName` carries Match-V5's id and,
    /// unlike `championName`, should not follow the client's language.
    pub(super) fn cell_champion(&self) -> Option<&str> {
        let raw = self
            .raw_champion_name
            .as_deref()
            .and_then(|raw| raw.strip_prefix(RAW_CHAMPION_PREFIX));
        raw.or(self.champion_name.as_deref())
    }

    pub(super) fn item_ids(&self) -> Vec<u32> {
        self.items.iter().map(|item| item.id).collect()
    }

    pub(super) fn item_counts(&self) -> ItemCounts {
        let mut counts = ItemCounts::new();
        for item in &self.items {
            *counts.entry(item.id).or_default() += item.count;
        }
        counts
    }
}
