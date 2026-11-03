use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// One way OpenAI may answer a request.
#[derive(Clone)]
pub enum Reply {
    Http {
        status: &'static str,
        headers: String,
        body: String,
    },
    /// Answers nothing until the caller gives up.
    Silence,
    /// Closes the connection unanswered, as a network error does.
    Hangup,
    /// Gives `reply` after `wait`.
    Delayed { wait: Duration, reply: Box<Self> },
}

impl Reply {
    pub fn status(status: &'static str) -> Self {
        Self::with_body(status, "{}".to_string())
    }

    pub const fn with_body(status: &'static str, body: String) -> Self {
        Self::Http {
            status,
            headers: String::new(),
            body,
        }
    }

    /// A Responses call whose model wrote `text`.
    pub fn advising(text: &str) -> Self {
        Self::with_body("200 OK", responses_body(text))
    }

    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        Self::Http {
            status: "429 Too Many Requests",
            headers: retry_after.map_or_else(String::new, |s| format!("retry-after: {s}\r\n")),
            body: error_body("rate_limit_exceeded"),
        }
    }

    pub fn out_of_quota() -> Self {
        Self::with_body("429 Too Many Requests", error_body("insufficient_quota"))
    }
}

/// Stands in for OpenAI: answers requests with `replies` in order, the last one for
/// every request after, and keeps the requests. A held fake answers only once released.
pub struct FakeOpenAi {
    pub url: String,
    requests: Arc<Mutex<Vec<(Instant, String)>>>,
    answered: Arc<AtomicUsize>,
    released: Arc<AtomicBool>,
}

impl FakeOpenAi {
    pub async fn answering(status: &'static str) -> Self {
        Self::replying(vec![Reply::status(status)]).await
    }

    pub async fn advising(text: &str) -> Self {
        Self::replying(vec![Reply::advising(text)]).await
    }

    /// As `advising`, but holds every answer until `release`.
    pub async fn advising_when_released(text: &str) -> Self {
        Self::serve(vec![Reply::advising(text)], false).await
    }

    pub async fn replying(replies: Vec<Reply>) -> Self {
        Self::serve(replies, true).await
    }

    async fn serve(replies: Vec<Reply>, released: bool) -> Self {
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
                let at = {
                    let mut requests = requests.lock().unwrap();
                    requests.push((Instant::now(), request));
                    requests.len() - 1
                };
                let reply = replies[at.min(replies.len() - 1)].clone();
                let (answered, gate) = (Arc::clone(&answered), Arc::clone(&gate));
                tokio::spawn(async move {
                    while !gate.load(Ordering::SeqCst) {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                    answer(stream, reply).await;
                    answered.fetch_add(1, Ordering::SeqCst);
                });
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
            .map(|(_, request)| request.to_lowercase())
            .collect()
    }

    /// The time between each request and the next.
    pub fn gaps(&self) -> Vec<Duration> {
        let requests = self.requests.lock().unwrap();
        requests
            .windows(2)
            .map(|pair| pair[1].0 - pair[0].0)
            .collect()
    }

    /// The JSON body of every request.
    pub fn bodies(&self) -> Vec<Value> {
        let requests = self.requests.lock().unwrap();
        requests
            .iter()
            .map(|(_, request)| {
                let body = request.split_once("\r\n\r\n").unwrap().1;
                serde_json::from_str(body).unwrap()
            })
            .collect()
    }

    /// Requests answered, or given up on by the caller.
    pub fn answered(&self) -> usize {
        self.answered.load(Ordering::SeqCst)
    }
}

async fn answer(mut stream: TcpStream, reply: Reply) {
    match reply {
        Reply::Http {
            status,
            headers,
            body,
        } => {
            let reply = format!(
                "HTTP/1.1 {status}\r\ncontent-length: {}\r\n{headers}connection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes()).await;
        }
        // The caller's timeout closes the connection, which ends the read.
        Reply::Silence => {
            let _ = stream.read(&mut [0; 1]).await;
        }
        Reply::Hangup => drop(stream),
        Reply::Delayed { wait, reply } => {
            tokio::time::sleep(wait).await;
            Box::pin(answer(stream, *reply)).await;
        }
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

fn error_body(code: &str) -> String {
    json!({
        "error": { "message": "fake", "type": code, "param": null, "code": code },
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
