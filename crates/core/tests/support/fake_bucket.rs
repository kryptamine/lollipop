use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use ed25519_dalek::{Signer, SigningKey};
use lollipop_core::delivery::{ManifestKey, TableChannel};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const SIGNING_SEED: [u8; 32] = [7; 32];

#[derive(Clone)]
enum Served {
    Whole(String),
    /// Promises more bytes than it sends, as a dropped connection does.
    CutShort(String),
}

#[derive(Default)]
struct State {
    manifest: Option<String>,
    tables: BTreeMap<u32, Served>,
    down: bool,
    manifest_downloads: usize,
    table_downloads: BTreeMap<u32, usize>,
    saw_credentials: bool,
}

/// Stands in for the public bucket: objects over plain GET, under the schema's path.
pub struct FakeBucket {
    url: String,
    state: Arc<Mutex<State>>,
}

impl FakeBucket {
    pub async fn start() -> Self {
        let state = Arc::new(Mutex::new(State::default()));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(answer(listener, Arc::clone(&state)));
        Self { url, state }
    }

    pub fn channel(&self) -> TableChannel {
        TableChannel::new(&self.url).unwrap()
    }

    /// Uploads the table and points the manifest at it.
    pub fn publish(&self, version: u32, table: &str) {
        self.upload(version, table);
        self.point_manifest_at(version, table);
    }

    pub fn upload(&self, version: u32, table: &str) {
        let served = Served::Whole(table.to_string());
        self.state.lock().unwrap().tables.insert(version, served);
    }

    /// The manifest names `table`, whatever the object of that version actually holds.
    pub fn point_manifest_at(&self, version: u32, table: &str) {
        let sha256 = sha256_of(table);
        let signature = sign_with_this_bucket(&message(1, version, &sha256));
        self.write_manifest(&manifest_body(version, &sha256, &signature));
    }

    pub fn publish_cut_short(&self, version: u32, table: &str) {
        self.point_manifest_at(version, table);
        let served = Served::CutShort(table.to_string());
        self.state.lock().unwrap().tables.insert(version, served);
    }

    pub fn write_manifest(&self, body: &str) {
        self.state.lock().unwrap().manifest = Some(body.to_string());
    }

    pub fn go_down(&self) {
        self.state.lock().unwrap().down = true;
    }

    pub fn manifest_downloads(&self) -> usize {
        self.state.lock().unwrap().manifest_downloads
    }

    pub fn table_downloads(&self, version: u32) -> usize {
        let state = self.state.lock().unwrap();
        state.table_downloads.get(&version).copied().unwrap_or(0)
    }

    pub fn saw_credentials(&self) -> bool {
        self.state.lock().unwrap().saw_credentials
    }
}

struct Reply {
    status: &'static str,
    body: String,
    promised_extra: usize,
}

impl Reply {
    const fn with_status(status: &'static str) -> Self {
        Self {
            status,
            body: String::new(),
            promised_extra: 0,
        }
    }

    fn ok(body: String) -> Self {
        Self {
            body,
            ..Self::with_status("200 OK")
        }
    }

    async fn send(self, stream: &mut TcpStream) {
        let head = format!(
            "HTTP/1.1 {}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
            self.status,
            self.body.len() + self.promised_extra
        );
        stream.write_all(head.as_bytes()).await.unwrap();
        stream.write_all(self.body.as_bytes()).await.unwrap();
    }
}

async fn read_request(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 4096];
    let read = stream.read(&mut buffer).await.unwrap();
    String::from_utf8_lossy(&buffer[..read]).to_lowercase()
}

async fn answer(listener: TcpListener, state: Arc<Mutex<State>>) {
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        let reply = reply_to(&request, &mut state.lock().unwrap());
        reply.send(&mut stream).await;
    }
}

fn reply_to(request: &str, state: &mut State) -> Reply {
    if state.down {
        return Reply::with_status("503 Service Unavailable");
    }
    state.saw_credentials |= request.contains("authorization: ");
    let path = request.split(' ').nth(1).unwrap_or_default();
    // Counted with the choice of body, so a test that saw the count can republish safely.
    if path == "/v1/manifest.json" {
        state.manifest_downloads += 1;
        return state
            .manifest
            .clone()
            .map_or(Reply::with_status("404 Not Found"), Reply::ok);
    }
    let Some(version) = path
        .strip_prefix("/v1/benchmark-table-")
        .and_then(|name| name.strip_suffix(".json"))
        .and_then(|version| version.parse().ok())
    else {
        return Reply::with_status("404 Not Found");
    };
    *state.table_downloads.entry(version).or_default() += 1;
    match state.tables.get(&version).cloned() {
        Some(Served::Whole(body)) => Reply::ok(body),
        Some(Served::CutShort(body)) => Reply {
            promised_extra: 100,
            ..Reply::ok(body)
        },
        None => Reply::with_status("404 Not Found"),
    }
}

/// The app's half of the key this bucket signs with.
pub fn manifest_key() -> ManifestKey {
    let public = SigningKey::from_bytes(&SIGNING_SEED).verifying_key();
    ManifestKey::from_hex(&hex::encode(public.as_bytes())).unwrap()
}

pub fn sha256_of(table: &str) -> String {
    format!("{:x}", Sha256::digest(table))
}

// Written out, not taken from the app, so that the format cannot drift unnoticed.
pub fn message(schema: u32, version: u32, sha256: &str) -> String {
    format!("lollipop-manifest\nv{schema}\n{version}\n{sha256}")
}

pub fn sign(seed: &[u8; 32], message: &str) -> String {
    let signature = SigningKey::from_bytes(seed).sign(message.as_bytes());
    hex::encode(signature.to_bytes())
}

pub fn sign_with_this_bucket(message: &str) -> String {
    sign(&SIGNING_SEED, message)
}

pub fn manifest_body(version: u32, sha256: &str, signature: &str) -> String {
    json!({ "version": version, "sha256": sha256, "signature": signature }).to_string()
}
