/// What made a Turning point.
pub(super) enum Moment {
    FirstTurret,
    /// A drake or the Elder, by `DragonType`.
    Dragon(Option<String>),
    Herald,
    Baron,
    Inhibitor,
    Ace,
}

impl Moment {
    pub(super) fn describe(&self) -> String {
        match self {
            Self::FirstTurret => "the laning phase ends: the first turret of the game fell".into(),
            Self::Dragon(Some(kind)) if kind == "Elder" => "the Elder Dragon was killed".into(),
            Self::Dragon(Some(kind)) => format!("a drake was killed: {kind}"),
            Self::Dragon(None) => "a drake was killed".into(),
            Self::Herald => "the Rift Herald was killed".into(),
            Self::Baron => "Baron Nashor was killed".into(),
            Self::Inhibitor => "an inhibitor was destroyed".into(),
            Self::Ace => "an ace: a whole team is dead".into(),
        }
    }
}
