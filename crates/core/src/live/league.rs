use std::path::Path;

/// Where League answers: the game's Live Client Data and the client's lockfile.
#[derive(Debug, Clone, Copy)]
pub struct League<'a> {
    pub live_client_url: &'a str,
    pub lockfile: &'a Path,
}
