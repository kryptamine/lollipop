use std::path::Path;

pub const LOCKFILE_PATH: &str = "/Applications/League of Legends.app/Contents/LoL/lockfile";

/// How to reach the League client, rewritten by the client on every launch.
pub(super) struct Lockfile {
    pub(super) port: u16,
    pub(super) password: String,
    pub(super) protocol: String,
}

impl Lockfile {
    pub(super) fn read(path: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        // name:pid:port:password:protocol
        let mut fields = text.trim().split(':').skip(2);
        Some(Self {
            port: fields.next()?.parse().ok()?,
            password: fields.next()?.to_string(),
            protocol: fields.next()?.to_string(),
        })
    }
}
