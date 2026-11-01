use std::process::Command;

use super::gh_error::GhError;

/// Where a run that nobody watches reports what stopped it.
pub(super) struct GhIssues<'a> {
    repo: &'a str,
}

impl<'a> GhIssues<'a> {
    pub(super) const fn new(repo: &'a str) -> Self {
        Self { repo }
    }

    pub(super) fn open(&self, title: &str, body: &str) -> Result<(), GhError> {
        self.run(&["issue", "create", "--title", title, "--body", body])
            .map(drop)
    }

    /// Opens at most one issue per title ever, so a closed one stays closed: the weekly
    /// build sees the same kill for as long as its patch stays stored.
    pub(super) fn open_once(&self, title: &str, body: &str) -> Result<(), GhError> {
        let search = format!("\"{title}\" in:title");
        let listed = self.run(&[
            "issue",
            "list",
            "--state",
            "all",
            "--search",
            &search,
            "--json",
            "title",
            "--jq",
            ".[].title",
        ])?;
        if listed.lines().any(|line| line == title) {
            return Ok(());
        }
        self.open(title, body)
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
