use super::models::Event;
use std::{io, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::Mutex,
};

pub struct Client {
    reader: Arc<Mutex<OwnedReadHalf>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn listen<F>(&self, mut callback: F)
    where
        F: FnMut(Event) + Send + 'static,
    {
        let mut temp_buffer = [0; 1024];

        loop {
            if let Ok(size) = self.reader.lock().await.read(&mut temp_buffer).await {
                if size > 0 {
                    let mut buffer = self.buffer.lock().await;
                    buffer.extend_from_slice(&temp_buffer[..size]);

                    while let Some(event_size) = Self::try_deserialize(&buffer) {
                        let event_data = buffer.drain(..event_size).collect::<Vec<_>>();
                        if let Ok(event) = bincode::deserialize::<Event>(&event_data) {
                            callback(event);
                        }
                    }
                }
            }
        }
    }

    pub async fn send(&self, event: &Event) -> io::Result<()> {
        let serialized = bincode::serialize(event)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Serialization failed"))?;
        let mut writer = self.writer.lock().await;
        writer.write_all(&serialized).await
    }

    fn try_deserialize(buffer: &[u8]) -> Option<usize> {
        match bincode::deserialize::<Event>(buffer) {
            Ok(_) => Some(buffer.len()),
            Err(err) => match *err {
                bincode::ErrorKind::SizeLimit => None,
                _ => None,
            },
        }
    }
}
