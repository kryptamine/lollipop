const TABLE_PREFIX: &str = "benchmark-table-";
const TABLE_SUFFIX: &str = ".json";
const KEPT_TABLES: usize = 4;

pub(super) const MANIFEST: &str = "manifest.json";

pub(super) fn table_name(version: u32) -> String {
    format!("{TABLE_PREFIX}{version}{TABLE_SUFFIX}")
}

/// What the bucket holds: the manifest, and the kept tables by version, oldest first.
pub(super) struct BucketAssets {
    has_manifest: bool,
    versions: Vec<u32>,
}

impl BucketAssets {
    pub(super) fn new(names: &str) -> Self {
        let mut versions: Vec<u32> = names.lines().filter_map(version_of).collect();
        versions.sort_unstable();
        Self {
            has_manifest: names.lines().any(|name| name == MANIFEST),
            versions,
        }
    }

    pub(super) const fn has_manifest(&self) -> bool {
        self.has_manifest
    }

    pub(super) const fn holds_tables(&self) -> bool {
        !self.versions.is_empty()
    }

    pub(super) fn holds(&self, version: u32) -> bool {
        self.versions.contains(&version)
    }

    /// Above every kept table, so a publish after a rollback never reuses a version.
    pub(super) fn next_version(&self) -> u32 {
        self.versions.last().map_or(1, |newest| newest + 1)
    }

    /// The tables to delete once a newer one is up.
    pub(super) fn outdated_after_adding_one(&self) -> &[u32] {
        let outdated = (self.versions.len() + 1).saturating_sub(KEPT_TABLES);
        self.versions.get(..outdated).unwrap_or(&self.versions)
    }
}

fn version_of(name: &str) -> Option<u32> {
    name.strip_prefix(TABLE_PREFIX)?
        .strip_suffix(TABLE_SUFFIX)?
        .parse()
        .ok()
}
