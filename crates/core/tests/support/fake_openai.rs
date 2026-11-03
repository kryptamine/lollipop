use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Stands in for OpenAI: answers every request with one status and keeps the requests.
pub struct FakeOpenAi {
    pub url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

impl FakeOpenAi {
    pub async fn answering(status: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::<Mutex<Vec<String>>>::default();
        let kept = Arc::clone(&requests);
        tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut buffer = [0; 4096];
                let read = stream.read(&mut buffer).await.unwrap();
                let request = String::from_utf8_lossy(&buffer[..read]).to_lowercase();
                kept.lock().unwrap().push(request);
                let reply = format!(
                    "HTTP/1.1 {status}\r\ncontent-length: 2\r\nconnection: close\r\n\r\n{{}}"
                );
                stream.write_all(reply.as_bytes()).await.unwrap();
            }
        });
        Self { url, requests }
    }

    /// The request line and headers of every request, lowercased.
    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}
