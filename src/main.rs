use net::{client::Client, models::Event, server::Server};
use std::time::Duration;
use tokio::{net::TcpStream, try_join};

mod net;

#[tokio::main]
async fn main() {
    if let Err(e) = try_join!(
        tokio::spawn(async move { run_server().await.unwrap() }),
        tokio::spawn(async move { run_client().await.unwrap() })
    ) {
        eprintln!("Error in main: {:?}", e);
    }
}

async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new("127.0.0.1:8909").await?;

    println!("Server started on 127.0.0.1:8909");

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
            tokio::time::sleep(Duration::from_secs(5)).await;
            let client_count = server.clients.lock().await.len();
            println!("Broadcasting event to {} clients", client_count);
    
            if let Err(e) = server
                .broadcast_event(Event {
                    opcode: 0,
                    data: "Hello from server".to_string(),
                })
                .await
            {
                eprintln!("Error broadcasting event: {:?}", e);
            }
        }
    });    

    server.start().await?;

    Ok(())
}

async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::sleep(Duration::from_secs(2)).await;

    let stream = TcpStream::connect("127.0.0.1:8909").await?;
    let client = Client::new(stream);

    println!("Client connected to server");

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

    client
        .send_event(&Event {
            opcode: 1,
            data: "Hello from client".to_string(),
        })
        .await?;

    println!("Client sent event");

    Ok(())
}
