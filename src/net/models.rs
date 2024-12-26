use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub opcode: u8,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloEvent {}

#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatEvent {}

#[derive(Debug)]
pub enum EventType {
    Hello(HelloEvent),
    Heartbeat(HeartbeatEvent),
}

pub fn deserialize(event: &Event) -> Option<EventType> {
    match event.opcode {
        0 => serde_json::from_str::<HelloEvent>(&event.data)
            .ok()
            .map(EventType::Hello),
        1 => serde_json::from_str::<HeartbeatEvent>(&event.data)
            .ok()
            .map(EventType::Heartbeat),
        _ => None,
    }
}
