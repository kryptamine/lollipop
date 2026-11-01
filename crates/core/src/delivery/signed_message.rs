use crate::benchmarks::SCHEMA_VERSION;

/// What a manifest's signature covers. The schema is part of it, so a manifest signed for
/// one schema's path cannot be served at another's.
#[must_use]
pub fn signed_message(version: u32, sha256: &str) -> String {
    format!("lollipop-manifest\nv{SCHEMA_VERSION}\n{version}\n{sha256}")
}
