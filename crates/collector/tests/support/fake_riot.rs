use std::collections::HashMap;
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::game::Game;

#[derive(Clone)]
pub struct Request {
    pub path: String,
    pub at_ms: u64,
    pub token: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Clone, Copy)]
pub struct Failure {
    pub status: u16,
    pub retry_after_s: Option<u64>,
}

#[derive(Default)]
struct State {
    seeds: HashMap<String, Vec<Value>>,
    match_ids: HashMap<String, Vec<String>>,
    games: HashMap<String, Game>,
    last_page: Option<u32>,
    rate_limit: Option<String>,
    failures: Vec<(String, Failure, u32)>,
    unreadable: Vec<String>,
    hang: Option<(String, usize)>,
    requests: Vec<Request>,
}

struct Reply {
    hangs: bool,
    status: u16,
    body: String,
    retry_after_s: Option<u64>,
    rate_limit: Option<String>,
}

/// Stands in for Riot's web API, on both the platform and the regional host.
#[derive(Clone)]
pub struct FakeRiot {
    pub url: String,
    state: Arc<Mutex<State>>,
}

impl FakeRiot {
    pub async fn start(now_ms: impl Fn() -> u64 + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(Mutex::new(State::default()));
        let server_state = Arc::clone(&state);
        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let reply_state = Arc::clone(&server_state);
                let at_ms = now_ms();
                tokio::spawn(async move {
                    let _ = answer(stream, reply_state, at_ms).await;
                });
            }
        });
        Self { url, state }
    }

    /// `tier` is a league name, so Master+ seeds go under `MASTER`, `GRANDMASTER` or
    /// `CHALLENGER`.
    pub fn add_seed(&self, tier: &str, puuid: &str, games: &[Game]) {
        let mut state = self.state.lock().unwrap();
        let seeds = state.seeds.entry(tier.to_string()).or_default();
        seeds.push(json!({ "puuid": puuid, "tier": tier, "summonerId": "not-a-puuid" }));
        let ids = games.iter().map(|game| game.id.clone()).collect();
        state.match_ids.insert(puuid.to_string(), ids);
        for game in games {
            state.games.insert(game.id.clone(), game.clone());
        }
    }

    /// Every division answers with its seeds on page 1 only, and with `[]` beyond it.
    pub fn have_one_page(&self) {
        self.state.lock().unwrap().last_page = Some(1);
    }

    pub fn set_rate_limit(&self, header: &str) {
        self.state.lock().unwrap().rate_limit = Some(header.to_string());
    }

    /// The next `times` requests whose path holds `path_part` fail, before any routing.
    pub fn fail(&self, path_part: &str, failure: Failure, times: u32) {
        let mut state = self.state.lock().unwrap();
        state.failures.push((path_part.to_string(), failure, times));
    }

    pub fn answer_unreadable(&self, path_part: &str) {
        let mut state = self.state.lock().unwrap();
        state.unreadable.push(path_part.to_string());
    }

    /// The call after the first `answered` holding `path_part` gets no answer until
    /// `release`, so a test can end a run at exactly that call.
    pub fn hang_after(&self, path_part: &str, answered: usize) {
        let mut state = self.state.lock().unwrap();
        state.hang = Some((path_part.to_string(), answered));
    }

    pub fn release(&self) {
        self.state.lock().unwrap().hang = None;
    }

    pub fn requests(&self) -> Vec<Request> {
        self.state.lock().unwrap().requests.clone()
    }

    pub fn request_count(&self, path_part: &str) -> usize {
        let requests = self.requests();
        let matching = requests.iter().filter(|r| r.path.contains(path_part));
        matching.count()
    }
}

async fn answer(
    mut stream: TcpStream,
    state: Arc<Mutex<State>>,
    at_ms: u64,
) -> std::io::Result<()> {
    let head = read_head(&mut stream).await?;
    // A client killed mid-request never made the call.
    if !head.ends_with("\r\n\r\n") {
        return Ok(());
    }
    let reply = state.lock().unwrap().reply_to(&head, at_ms);
    if reply.hangs {
        while state.lock().unwrap().hang.is_some() {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    let mut response = format!(
        "HTTP/1.1 {} X\r\ncontent-length: {}\r\nconnection: close\r\n",
        reply.status,
        reply.body.len()
    );
    if let Some(limit) = reply.rate_limit {
        write!(response, "x-app-rate-limit: {limit}\r\n").unwrap();
    }
    if let Some(seconds) = reply.retry_after_s {
        write!(response, "retry-after: {seconds}\r\n").unwrap();
    }
    response.push_str("\r\n");
    response.push_str(&reply.body);
    stream.write_all(response.as_bytes()).await
}

async fn read_head(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut head = Vec::new();
    let mut chunk = [0; 4096];
    while !head.windows(4).any(|end| end == b"\r\n\r\n") {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        head.extend_from_slice(&chunk[..read]);
    }
    Ok(String::from_utf8_lossy(&head).into_owned())
}

fn header(head: &str, name: &str) -> Option<String> {
    head.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.eq_ignore_ascii_case(name)
            .then(|| value.trim().to_string())
    })
}

impl State {
    fn reply_to(&mut self, head: &str, at_ms: u64) -> Reply {
        let path = head
            .split_whitespace()
            .nth(1)
            .unwrap_or_default()
            .to_string();
        self.requests.push(Request {
            path: path.clone(),
            at_ms,
            token: header(head, "x-riot-token"),
            user_agent: header(head, "user-agent"),
        });
        let rate_limit = self.rate_limit.clone();
        let hangs = self.hanging_on(&path);
        if let Some(failure) = self.take_failure(&path) {
            return Reply {
                hangs,
                status: failure.status,
                body: String::new(),
                retry_after_s: failure.retry_after_s,
                rate_limit,
            };
        }
        let body = self.route(&path);
        Reply {
            hangs,
            status: if body.is_some() { 200 } else { 404 },
            body: body.unwrap_or_default(),
            retry_after_s: None,
            rate_limit,
        }
    }

    fn hanging_on(&self, path: &str) -> bool {
        self.hang.as_ref().is_some_and(|(part, answered)| {
            let seen = self
                .requests
                .iter()
                .filter(|r| r.path.contains(part.as_str()));
            path.contains(part.as_str()) && seen.count() > *answered
        })
    }

    fn take_failure(&mut self, path: &str) -> Option<Failure> {
        let entry = self
            .failures
            .iter_mut()
            .find(|(part, _, times)| *times > 0 && path.contains(part.as_str()))?;
        entry.2 -= 1;
        Some(entry.1)
    }

    fn route(&self, full_path: &str) -> Option<String> {
        if self
            .unreadable
            .iter()
            .any(|part| full_path.contains(part.as_str()))
        {
            return Some(String::from("<html>not json</html>"));
        }
        let (path, query) = full_path.split_once('?').unwrap_or((full_path, ""));
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        match parts.as_slice() {
            ["lol", "league", "v4", "entries", _, tier, _] => Some(self.entries(tier, query)),
            ["lol", "league", "v4", league, "by-queue", _] => Some(self.apex(league)),
            ["lol", "match", "v5", "matches", "by-puuid", puuid, "ids"] => {
                self.match_ids.get(*puuid).map(|ids| json!(ids).to_string())
            }
            ["lol", "match", "v5", "matches", id, "timeline"] => {
                let game = self.games.get(*id).filter(|game| game.has_timeline)?;
                Some(game.timeline_body().to_string())
            }
            ["lol", "match", "v5", "matches", id] => {
                Some(self.games.get(*id)?.match_body().to_string())
            }
            _ => None,
        }
    }

    fn entries(&self, tier: &str, query: &str) -> String {
        let page: u32 = query.trim_start_matches("page=").parse().unwrap();
        if self.last_page.is_some_and(|last| page > last) {
            return String::from("[]");
        }
        json!(self.seeds.get(tier).cloned().unwrap_or_default()).to_string()
    }

    fn apex(&self, league: &str) -> String {
        let tier = league.trim_end_matches("leagues").to_ascii_uppercase();
        let entries = self.seeds.get(&tier).cloned().unwrap_or_default();
        json!({ "tier": tier, "entries": entries }).to_string()
    }
}
