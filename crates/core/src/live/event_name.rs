use serde::Deserialize;

/// The `EventName`s Lollipop reads. Any other is `Other`, so a new one never fails the poll.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
pub(super) enum EventName {
    GameStart,
    GameEnd,
    ChampionKill,
    DragonKill,
    HeraldKill,
    /// Voidgrubs.
    HordeKill,
    BaronKill,
    TurretKilled,
    InhibKilled,
    Ace,
    #[serde(other)]
    Other,
}
