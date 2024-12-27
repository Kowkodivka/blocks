use std::time::Duration;

use net::{client::Client, models::Event, server::Server};
use tokio::{net::TcpStream, try_join};

mod net;

#[tokio::main]
async fn main() {
    try_join!(
        tokio::spawn(async move {
            run_server().await.unwrap();
        }),
        tokio::spawn(async move {
            run_client().await.unwrap();
        })
    )
    .unwrap();
}

async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new("127.0.0.1:8909").await?;
    println!("Server started on 127.0.0.1:8909");

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone
            .listen_events(|event| {
                println!("Server received event: {:?}", event);
            })
            .await;
    });

    // let server_clone = server.clone();
    // tokio::spawn(async move {
    //     tokio::time::sleep(Duration::from_secs(5)).await;

    //     server_clone
    //         .broadcast_event(Event {
    //             opcode: 0,
    //             data: "Hello from client".to_string(),
    //         })
    //         .await
    //         .unwrap();
    // });

    server.start().await?;
    Ok(())
}

async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::sleep(Duration::from_secs(2)).await;

    let stream = TcpStream::connect("127.0.0.1:8909").await?;
    let client = Client::new(stream);
    println!("Client connected to server");

    let client_clone = client.clone();
    tokio::spawn(async move {
        if let Err(e) = client_clone
            .listen(|event| {
                println!("Client received event: {:?}", event);
            })
            .await
        {
            eprintln!("Error listening to events on client: {:?}", e);
        }
    });

    client
        .send_event(&Event {
            opcode: 0,
            data: "Hello from client".to_string(),
        })
        .await?;

    println!("Client sent event");

    Ok(())
}
