use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Stands in for OpenAI: answers every request with one status and body, and keeps
/// the requests. A held fake answers only once released.
pub struct FakeOpenAi {
    pub url: String,
    requests: Arc<Mutex<Vec<String>>>,
    answered: Arc<AtomicUsize>,
    released: Arc<AtomicBool>,
}

impl FakeOpenAi {
    pub async fn answering(status: &'static str) -> Self {
        Self::serve(status, "{}".to_string(), true).await
    }

    /// Answers a Responses call whose model wrote `text`.
    pub async fn advising(text: &str) -> Self {
        Self::serve("200 OK", responses_body(text), true).await
    }

    /// As `advising`, but holds every answer until `release`.
    pub async fn advising_when_released(text: &str) -> Self {
        Self::serve("200 OK", responses_body(text), false).await
    }

    async fn serve(status: &'static str, body: String, released: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let fake = Self {
            url: format!("http://{}", listener.local_addr().unwrap()),
            requests: Arc::default(),
            answered: Arc::default(),
            released: Arc::new(AtomicBool::new(released)),
        };
        let requests = Arc::clone(&fake.requests);
        let answered = Arc::clone(&fake.answered);
        let gate = Arc::clone(&fake.released);
        tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_request(&mut stream).await;
                requests.lock().unwrap().push(request);
                while !gate.load(Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                let reply = format!(
                    "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(reply.as_bytes()).await.unwrap();
                answered.fetch_add(1, Ordering::SeqCst);
            }
        });
        fake
    }

    pub fn release(&self) {
        self.released.store(true, Ordering::SeqCst);
    }

    /// Every request, head and body, lowercased.
    pub fn requests(&self) -> Vec<String> {
        let requests = self.requests.lock().unwrap();
        requests
            .iter()
            .map(|request| request.to_lowercase())
            .collect()
    }

    /// The JSON body of every request.
    pub fn bodies(&self) -> Vec<Value> {
        let requests = self.requests.lock().unwrap();
        requests
            .iter()
            .map(|request| {
                let body = request.split_once("\r\n\r\n").unwrap().1;
                serde_json::from_str(body).unwrap()
            })
            .collect()
    }

    pub fn answered(&self) -> usize {
        self.answered.load(Ordering::SeqCst)
    }
}

/// A `POST /v1/responses` reply carrying the model's `text`.
pub fn responses_body(text: &str) -> String {
    json!({
        "id": "resp_fake",
        "status": "completed",
        "output": [{
            "type": "message",
            "role": "assistant",
            "content": [{ "type": "output_text", "text": text, "annotations": [] }],
        }],
    })
    .to_string()
}

// Reads the head, then as much body as `content-length` says.
async fn read_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        let read = stream.read(&mut buffer).await.unwrap();
        bytes.extend_from_slice(&buffer[..read]);
        let text = String::from_utf8_lossy(&bytes).to_string();
        if read == 0 {
            return text;
        }
        let Some((head, body)) = text.split_once("\r\n\r\n") else {
            continue;
        };
        let length = head
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().unwrap())
            })
            .unwrap_or(0);
        if body.len() >= length {
            return text;
        }
    }
}
