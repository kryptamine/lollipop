use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use ed25519_dalek::{Signer, SigningKey};
use lollipop_core::delivery::{ManifestKey, ReleaseChannel, ReleaseToken};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub const TOKEN: &str = "test-token";
const REPO: &str = "owner/private-repo";
const SIGNING_SEED: [u8; 32] = [7; 32];
const MANIFEST_ID: u32 = 1;
// An asset id of a table is this plus its version.
const TABLE_IDS: u32 = 1000;

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
    token_at_storage: bool,
}

/// Stands in for GitHub: the REST API, and the storage host its asset downloads redirect to.
/// As the real ones do, the API wants a user agent and the token, and storage refuses a token.
pub struct FakeRelease {
    api: String,
    state: Arc<Mutex<State>>,
}

impl FakeRelease {
    pub async fn start() -> Self {
        let state = Arc::new(Mutex::new(State::default()));
        let storage = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let storage_url = format!("http://{}", storage.local_addr().unwrap());
        tokio::spawn(answer_storage(storage, Arc::clone(&state)));
        let api = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let api_url = format!("http://{}", api.local_addr().unwrap());
        tokio::spawn(answer_api(api, Arc::clone(&state), storage_url));
        Self {
            api: api_url,
            state,
        }
    }

    pub fn channel(&self) -> ReleaseChannel {
        self.channel_with(ReleaseToken::new(TOKEN))
    }

    pub fn channel_with(&self, token: ReleaseToken) -> ReleaseChannel {
        ReleaseChannel::new(&self.api, REPO, token).unwrap()
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

    /// The manifest names `table`, whatever the asset of that version actually holds.
    pub fn point_manifest_at(&self, version: u32, table: &str) {
        let sha256 = sha256_of(table);
        let signature = sign_with_this_release(&message(1, version, &sha256));
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

    pub fn storage_saw_a_token(&self) -> bool {
        self.state.lock().unwrap().token_at_storage
    }
}

struct Reply {
    status: &'static str,
    location: Option<String>,
    body: String,
    promised_extra: usize,
}

impl Reply {
    const fn with_status(status: &'static str) -> Self {
        Self {
            status,
            location: None,
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
        let location = self
            .location
            .map(|url| format!("location: {url}\r\n"))
            .unwrap_or_default();
        let head = format!(
            "HTTP/1.1 {}\r\n{location}content-length: {}\r\nconnection: close\r\n\r\n",
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

fn path_of(request: &str) -> &str {
    request.split(' ').nth(1).unwrap_or_default()
}

async fn answer_api(listener: TcpListener, state: Arc<Mutex<State>>, storage: String) {
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        let reply = api_reply(&request, &state.lock().unwrap(), &storage);
        reply.send(&mut stream).await;
    }
}

fn api_reply(request: &str, state: &State, storage: &str) -> Reply {
    if state.down {
        return Reply::with_status("503 Service Unavailable");
    }
    if !request.contains("user-agent: ") {
        return Reply::with_status("403 Forbidden");
    }
    // A private repository answers a bad token as if it did not exist.
    if !request.contains(&format!("authorization: bearer {TOKEN}\r\n")) {
        return Reply::with_status("404 Not Found");
    }
    let path = path_of(request);
    let releases = format!("/repos/{REPO}/releases");
    if path == format!("{releases}/tags/v1") {
        return Reply::ok(release_listing(state));
    }
    let Some(id) = path
        .strip_prefix(&format!("{releases}/assets/"))
        .and_then(|id| id.parse::<u32>().ok())
    else {
        return Reply::with_status("404 Not Found");
    };
    // Without this header GitHub answers with the asset's metadata, not its bytes.
    if !request.contains("accept: application/octet-stream") {
        return Reply::ok(r#"{ "id": 1 }"#.to_string());
    }
    Reply {
        location: Some(format!("{storage}/{id}")),
        ..Reply::with_status("302 Found")
    }
}

fn release_listing(state: &State) -> String {
    let mut assets = vec![json!({ "id": MANIFEST_ID, "name": "manifest.json" })];
    for version in state.tables.keys() {
        let name = format!("benchmark-table-{version}.json");
        assets.push(json!({ "id": TABLE_IDS + version, "name": name }));
    }
    json!({ "assets": assets }).to_string()
}

async fn answer_storage(listener: TcpListener, state: Arc<Mutex<State>>) {
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        let reply = storage_reply(&request, &mut state.lock().unwrap());
        reply.send(&mut stream).await;
    }
}

fn storage_reply(request: &str, state: &mut State) -> Reply {
    if request.contains("authorization: ") {
        state.token_at_storage = true;
        return Reply::with_status("400 Bad Request");
    }
    let id: u32 = path_of(request)
        .trim_start_matches('/')
        .parse()
        .unwrap_or_default();
    // Counted with the choice of body, so a test that saw the count can republish safely.
    if id == MANIFEST_ID {
        state.manifest_downloads += 1;
        return Reply::ok(state.manifest.clone().unwrap_or_default());
    }
    *state.table_downloads.entry(id - TABLE_IDS).or_default() += 1;
    match state.tables.get(&(id - TABLE_IDS)).cloned() {
        Some(Served::Whole(body)) => Reply::ok(body),
        Some(Served::CutShort(body)) => Reply {
            promised_extra: 100,
            ..Reply::ok(body)
        },
        None => Reply::with_status("404 Not Found"),
    }
}

/// The app's half of the key this Release signs with.
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

pub fn sign_with_this_release(message: &str) -> String {
    sign(&SIGNING_SEED, message)
}

pub fn manifest_body(version: u32, sha256: &str, signature: &str) -> String {
    json!({ "version": version, "sha256": sha256, "signature": signature }).to_string()
}
