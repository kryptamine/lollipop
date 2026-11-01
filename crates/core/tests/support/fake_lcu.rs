use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub const SESSION: &str = "/lol-gameflow/v1/session";
pub const VERSION: &str = "/lol-patch/v1/game-version";
pub const RANKED: &str = "/lol-ranked/v1/current-ranked-stats";
pub const CHAMP_SELECT: &str = "/lol-champ-select/v1/session";

const PASSWORD: &str = "secret";
// Base64 of riot:secret.
const CREDENTIALS: &str = "Basic cmlvdDpzZWNyZXQ=";

type Routes = Arc<Mutex<HashMap<String, String>>>;

/// Stands in for the League client: a lockfile and an HTTP server with the password
/// it names. A path with no route answers 404, as the client's do.
pub struct FakeLcu {
    dir: TempDir,
    routes: Routes,
    requests: Arc<Mutex<Vec<String>>>,
}

impl FakeLcu {
    /// In queue 420 on patch 16.21 at Gold, with no champ select session.
    pub async fn start() -> Self {
        let lcu = Self::empty().await;
        lcu.set(SESSION, r#"{ "gameData": { "queue": { "id": 420 } } }"#);
        lcu.set(
            VERSION,
            r#""16.21.8342754+branch.releases-16-21.code.public""#,
        );
        lcu.set(RANKED, &ranked("GOLD"));
        lcu
    }

    pub async fn empty() -> Self {
        Self::with_password(PASSWORD).await
    }

    /// The lockfile gives `password`, while the server still asks for its own.
    pub async fn with_password(password: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let dir = TempDir::new().unwrap();
        // The real file names https; plain http spares the fake a certificate.
        let line = format!("LeagueClient:1234:{port}:{password}:http");
        std::fs::write(dir.path().join("lockfile"), line).unwrap();

        let routes = Routes::default();
        let requests = Arc::new(Mutex::new(Vec::new()));
        tokio::spawn(answer(listener, Arc::clone(&routes), Arc::clone(&requests)));
        Self {
            dir,
            routes,
            requests,
        }
    }

    pub fn lockfile(&self) -> PathBuf {
        self.dir.path().join("lockfile")
    }

    pub fn set(&self, path: &str, body: &str) {
        let mut routes = self.routes.lock().unwrap();
        routes.insert(path.to_string(), body.to_string());
    }

    pub fn remove(&self, path: &str) {
        self.routes.lock().unwrap().remove(path);
    }

    pub fn requests_for(&self, path: &str) -> usize {
        let requests = self.requests.lock().unwrap();
        requests.iter().filter(|seen| *seen == path).count()
    }
}

pub fn ranked(solo_duo_tier: &str) -> String {
    serde_json::json!({ "queues": [
        { "queueType": "RANKED_FLEX_SR", "tier": "CHALLENGER" },
        { "queueType": "RANKED_SOLO_5x5", "tier": solo_duo_tier },
    ] })
    .to_string()
}

pub fn champ_select(assigned_position: &str) -> String {
    serde_json::json!({
        "localPlayerCellId": 3,
        "myTeam": [
            { "cellId": 2, "assignedPosition": "top" },
            { "cellId": 3, "assignedPosition": assigned_position },
        ],
    })
    .to_string()
}

/// A lockfile whose client has gone: nothing listens on its port.
pub async fn stale_lockfile(dir: &Path) -> PathBuf {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let path = dir.join("lockfile");
    std::fs::write(&path, format!("LeagueClient:1234:{port}:{PASSWORD}:http")).unwrap();
    path
}

async fn answer(listener: TcpListener, routes: Routes, requests: Arc<Mutex<Vec<String>>>) {
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buffer = [0; 4096];
        let read = stream.read(&mut buffer).await.unwrap();
        let request = String::from_utf8_lossy(&buffer[..read]).to_string();
        let path = request.split(' ').nth(1).unwrap_or_default().to_string();

        let route = routes.lock().unwrap().get(&path).cloned();
        let (status, body) = if !request.contains(CREDENTIALS) {
            ("401 Unauthorized", String::new())
        } else if let Some(body) = route {
            ("200 OK", body)
        } else {
            ("404 Not Found", String::new())
        };
        requests.lock().unwrap().push(path);
        let head = format!(
            "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(head.as_bytes()).await.unwrap();
        stream.write_all(body.as_bytes()).await.unwrap();
    }
}
