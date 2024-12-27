use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub opcode: u8,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloEvent;

#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatEvent;

#[derive(Debug)]
pub enum EventType {
    Hello(HelloEvent),
    Heartbeat(HeartbeatEvent),
    Unknown(u8, String),
}

impl EventType {
    pub fn from_event(event: &Event) -> Self {
        match event.opcode {
            0 => serde_json::from_str::<HelloEvent>(&event.data)
                .map(EventType::Hello)
                .unwrap_or_else(|_| EventType::Unknown(event.opcode, event.data.clone())),
            1 => serde_json::from_str::<HeartbeatEvent>(&event.data)
                .map(EventType::Heartbeat)
                .unwrap_or_else(|_| EventType::Unknown(event.opcode, event.data.clone())),
            _ => EventType::Unknown(event.opcode, event.data.clone()),
        }
    }
}
