use super::models::Event;
use std::sync::Arc;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{Mutex, Notify},
};

pub struct Client {
    stream: Arc<Mutex<TcpStream>>,
    buffer: Arc<Mutex<Vec<u8>>>,
    read_notify: Arc<Notify>,
    paused: Arc<Mutex<bool>>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
            buffer: Arc::new(Mutex::new(Vec::new())),
            read_notify: Arc::new(Notify::new()),
            paused: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn listen<F>(&self, mut callback: F) -> io::Result<()>
    where
        F: FnMut(Event) + Send + 'static,
    {
        let mut temp_buffer = [0; 1024];

        loop {
            {
                let is_paused = *self.paused.lock().await;
                if is_paused {
                    self.read_notify.notified().await;
                    continue;
                }
            }

            let size = {
                let mut stream = self.stream.lock().await;
                stream.read(&mut temp_buffer).await?
            };

            if size == 0 {
                break; // Соединение закрыто
            }

            let mut buffer = self.buffer.lock().await;
            buffer.extend_from_slice(&temp_buffer[..size]);

            while let Some(event_size) = Self::try_deserialize(&buffer) {
                let event_data = buffer.drain(..event_size).collect::<Vec<_>>();
                if let Ok(event) = bincode::deserialize::<Event>(&event_data) {
                    callback(event);
                }
            }
        }

        Ok(())
    }

    pub async fn send_event(&self, event: &Event) -> io::Result<()> {
        self.pause_reading().await;

        let result = {
            let serialized = bincode::serialize(event)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Serialization failed"))?;

            let mut stream = self.stream.lock().await;
            stream.write_all(&serialized).await
        };

        self.resume_reading().await;

        result
    }

    pub async fn is_connected(&self) -> bool {
        self.stream.lock().await.peer_addr().is_ok()
    }

    async fn pause_reading(&self) {
        let mut paused = self.paused.lock().await;
        *paused = true;
    }

    async fn resume_reading(&self) {
        {
            let mut paused = self.paused.lock().await;
            *paused = false;
        }
        self.read_notify.notify_one();
    }

    fn try_deserialize(buffer: &[u8]) -> Option<usize> {
        match bincode::deserialize::<Event>(buffer) {
            Ok(_) => Some(buffer.len()), // Вернуть длину десериализованного объекта
            Err(err) => match *err {
                bincode::ErrorKind::SizeLimit => None, // Недостаточно данных
                _ => None,                             // Другие ошибки
            },
        }
    }
}
