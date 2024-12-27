use super::models::Event;
use std::sync::Arc;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

#[derive(Clone)]
pub struct Client {
    inner: Arc<Mutex<ClientInner>>,
}

struct ClientInner {
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClientInner {
                stream,
                buffer: Vec::new(),
            })),
        }
    }

    pub async fn send_event(&self, event: &Event) -> io::Result<()> {
        let mut inner = self.inner.lock().await;
        let serialized = bincode::serialize(event).unwrap();
        inner.stream.write_all(&serialized).await
    }

    pub async fn listen<F>(&self, mut callback: F) -> io::Result<()>
    where
        F: FnMut(Event) + Send + 'static,
    {
        let mut inner = self.inner.lock().await;
        loop {
            let mut temp_buffer = [0; 1024];
            let size = inner.stream.read(&mut temp_buffer).await?;

            if size == 0 {
                break;
            }

            inner.buffer.extend_from_slice(&temp_buffer[..size]);

            if let Ok(event) = bincode::deserialize::<Event>(&inner.buffer) {
                inner.buffer.clear();
                callback(event);
            }
        }

        Ok(())
    }
}
