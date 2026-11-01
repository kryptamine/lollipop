use std::path::Path;
use std::process::Command;

use super::gh_error::GhError;
use super::release_assets::ReleaseAssets;
use crate::table::SCHEMA_VERSION;

// `gh` prints exactly this, and exits with the same code as for an outage.
const NO_RELEASE: &str = "release not found";

/// The Release of one schema: its tag is the `v1` of the `v1/manifest.json` path.
pub(super) struct GhRelease<'a> {
    repo: &'a str,
    tag: String,
}

impl<'a> GhRelease<'a> {
    pub(super) fn new(repo: &'a str) -> Self {
        Self {
            repo,
            tag: format!("v{SCHEMA_VERSION}"),
        }
    }

    /// `None` when the Release does not exist yet.
    pub(super) fn assets(&self) -> Result<Option<ReleaseAssets>, GhError> {
        let listed = self.run(&[
            "release",
            "view",
            &self.tag,
            "--json",
            "assets",
            "--jq",
            ".assets[].name",
        ]);
        match listed {
            Ok(names) => Ok(Some(ReleaseAssets::new(&names))),
            Err(error) if error.reason == NO_RELEASE => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub(super) fn create(&self) -> Result<(), GhError> {
        self.run(&[
            "release",
            "create",
            &self.tag,
            "--title",
            "Benchmark tables",
            "--notes",
            "Written by lollipop-collector publish. manifest.json names the live table.",
            "--latest=false",
        ])
        .map(drop)
    }

    pub(super) fn download(&self, asset: &str, to: &Path) -> Result<(), GhError> {
        self.run(&[
            "release",
            "download",
            &self.tag,
            "--pattern",
            asset,
            "--output",
            &to.to_string_lossy(),
            "--clobber",
        ])
        .map(drop)
    }

    pub(super) fn upload(&self, file: &Path) -> Result<(), GhError> {
        self.run(&[
            "release",
            "upload",
            &self.tag,
            &file.to_string_lossy(),
            "--clobber",
        ])
        .map(drop)
    }

    pub(super) fn delete_asset(&self, asset: &str) -> Result<(), GhError> {
        self.run(&["release", "delete-asset", &self.tag, asset, "--yes"])
            .map(drop)
    }

    pub(super) fn open_issue(&self, title: &str, body: &str) -> Result<(), GhError> {
        self.run(&["issue", "create", "--title", title, "--body", body])
            .map(drop)
    }

    fn run(&self, args: &[&str]) -> Result<String, GhError> {
        let command = args.iter().take(2).copied().collect::<Vec<_>>().join(" ");
        let output = Command::new("gh")
            .args(args)
            .args(["--repo", self.repo])
            .output()
            .map_err(|error| GhError {
                command: command.clone(),
                reason: error.to_string(),
            })?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        Err(GhError {
            command,
            reason: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}
