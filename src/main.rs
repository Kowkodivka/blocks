use net::{client::Client, models::Event, server::Server};
use std::sync::Arc;
use tokio::{net::TcpStream, spawn, time::Duration};

mod net;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let addr = "127.0.0.1:8080";

    let server = Server::new(addr).await?;

    let server_handle = {
        let server = server.clone();
        spawn(async move {
            server.start().await.unwrap();
        })
    };

    tokio::time::sleep(Duration::from_secs(1)).await;

    let client_stream = TcpStream::connect(addr).await?;
    let client = Arc::new(Client::new(client_stream));

    let server_events = {
        let server = server.clone();
        spawn(async move {
            server
                .listen_events(|event| {
                    println!("Сервер получил событие: {:?}", event);
                })
                .await;
        })
    };

    let client_events = {
        let client = client.clone();
        spawn(async move {
            client
                .listen(|event| {
                    println!("Клиент получил событие: {:?}", event);
                })
                .await;
        })
    };

    let server_to_client = {
        let server = server.clone();
        spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let event = Event {
                opcode: 0,
                data: "Hello from server".to_string(),
            };
            server.broadcast_event(event).await;
        })
    };

    let client_to_server = {
        let client = client.clone();
        spawn(async move {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let event = Event {
                opcode: 0,
                data: "Hello from client".to_string(),
            };
            if let Err(e) = client.send(&event).await {
                eprintln!("Ошибка отправки события от клиента: {:?}", e);
            }
        })
    };

    tokio::try_join!(
        server_handle,
        server_events,
        client_events,
        server_to_client,
        client_to_server
    )?;

    Ok(())
}
