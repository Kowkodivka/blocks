use std::time::Duration;

use net::{
    client::Client,
    models::{Event, HelloEvent},
    server::Server,
};
use tokio::net::TcpStream;

mod net;

// TODO:
// 1. Clean structure
// 2. Event bus
// 3. Authorization
// 4. Sessions

#[tokio::main]
async fn main() {
    let server = tokio::spawn(async move {
        let server = Server::new("127.0.0.1:12345").await;
        server.start().await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    let client = tokio::spawn(async move {
        match TcpStream::connect("127.0.0.1:12345").await {
            Ok(stream) => {
                let mut client = Client::new(stream);
                let event = Event {
                    opcode: 0,
                    data: serde_json::to_string(&HelloEvent {}).unwrap(),
                };
                client.send_event(&event).await;
                println!("Event sent");
            }
            Err(e) => {
                eprintln!("Failed to connect to server: {:?}", e);
            }
        }
    });

    tokio::try_join!(server, client).unwrap();
}
