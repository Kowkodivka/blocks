use super::models::Event;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

pub struct TcpClient {
    reader: Arc<Mutex<TcpStream>>,
    writer: Arc<Mutex<TcpStream>>,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl TcpClient {
    pub fn new(stream: TcpStream) -> Self {
        let reader = stream.try_clone().expect("Failed to clone stream");
        let writer = stream;
        Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn listen<F>(&self, mut callback: F)
    where
        F: FnMut(Event),
    {
        let mut temp_buffer = [0; 1024];
        loop {
            let size = {
                let mut reader = self.reader.lock().unwrap();
                reader.read(&mut temp_buffer).unwrap_or(0)
            };

            if size > 0 {
                let mut buffer = self.buffer.lock().unwrap();
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

    pub fn send(&self, event: &Event) -> io::Result<()> {
        let serialized = bincode::serialize(event)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Serialization failed"))?;
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(&serialized)
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
