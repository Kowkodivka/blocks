use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use super::models::Event;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub async fn send_event(&mut self, event: &Event) {
        let serialized = bincode::serialize(event).unwrap();
        self.stream.write_all(&serialized).await.unwrap();
    }

    pub async fn receive_event(&mut self) -> Option<Event> {
        let mut buffer = [0; 1024];

        let size = self.stream.read(&mut buffer).await.unwrap_or(0);

        if size == 0 {
            return None;
        }

        let event: Event = bincode::deserialize(&buffer[..size]).unwrap();

        Some(event)
    }
}
