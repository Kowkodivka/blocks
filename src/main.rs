use std::time::Duration;

use net::{client::Client, models::Event, server::Server};
use tokio::{net::TcpStream, try_join};

mod net;

#[tokio::main]
async fn main() {
    try_join!(
        tokio::spawn(async move {
            let server = Server::new("127.0.0.1:8909").await.unwrap();

            tokio::spawn({
                let server = server.clone();
                async move {
                    server
                        .listen_events(|event| {
                            println!("Server received event: {:?}", event);
                        })
                        .await;
                }
            });

            tokio::spawn({
                let server = server.clone();
                async move {
                    tokio::time::sleep(Duration::from_secs(10)).await;

                    server
                        .broadcast_event(Event {
                            opcode: 0,
                            data: "Hello from server".to_string(),
                        })
                        .await
                        .unwrap();
                }
            });

            server.start().await.unwrap();
        }),
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if let Ok(stream) = TcpStream::connect("127.0.0.1:8909").await {
                let client = Client::new(stream);

                tokio::spawn({
                    let client = client.clone();
                    async move {
                        if let Err(e) = client
                            .listen(|event| {
                                println!("Client received event: {:?}", event);
                            })
                            .await
                        {
                            eprintln!("Error listening to events on client: {:?}", e);
                        }
                    }
                });

                tokio::spawn({
                    let client = client.clone();
                    async move {
                        client
                            .send_event(&Event {
                                opcode: 0,
                                data: "Hello from client".to_string(),
                            })
                            .await
                            .unwrap();
                    }
                });
            }
        })
    )
    .unwrap();
}
