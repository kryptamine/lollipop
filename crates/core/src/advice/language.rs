/// The language Advice is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Russian,
}

impl Language {
    /// Reads a BCP 47 tag such as `ru-RU`. Any language Advice is not written in,
    /// or none at all, gives English.
    #[must_use]
    pub fn of_locale(locale: Option<&str>) -> Self {
        let primary = locale
            .and_then(|tag| tag.split(['-', '_']).next())
            .map(str::to_ascii_lowercase);
        if primary.as_deref() == Some("ru") {
            Self::Russian
        } else {
            Self::English
        }
    }

    /// How the prompt names it.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Russian => "Russian",
        }
    }
}
