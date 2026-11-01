use std::fmt;

#[derive(Debug)]
pub enum Published {
    New(u32),
    /// The build matches the manifest's table, so it takes no slot among the kept ones.
    Unchanged(u32),
}

impl fmt::Display for Published {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::New(version) => write!(formatter, "published table {version}"),
            Self::Unchanged(version) => write!(formatter, "table {version} is unchanged"),
        }
    }
}
