use bevy::prelude::*;
use net::{client::TcpClient, models::Event, server::TcpServer};
use std::{net::TcpStream, sync::Arc, thread, time::Duration};

mod net;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_server, setup_client))
        .run();
}

fn setup_server() {
    thread::spawn(|| {
        let addr = "127.0.0.1:8080";
        let server = TcpServer::new(addr).unwrap();

        // Поток для запуска сервера
        let server_handle = {
            let server = server.clone();
            thread::spawn(move || {
                info!("Сервер запущен на {}", addr);
                server.start().unwrap();
            })
        };

        // Поток для обработки событий от клиентов
        let server_events = {
            let server = server.clone();
            thread::spawn(move || {
                server.listen_events(|event| {
                    info!("Сервер получил событие: {:?}", event);
                });
            })
        };

        // Поток для отправки события клиентам
        let server_to_client = {
            let server = server.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(5));
                let event = Event {
                    opcode: 0,
                    data: "Hello from server".to_string(),
                };
                server.broadcast_event(event);
            })
        };

        server_handle.join().unwrap();
        server_events.join().unwrap();
        server_to_client.join().unwrap();
    });
}

fn setup_client() {
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(2));

        let addr = "127.0.0.1:8080";
        let client_stream = TcpStream::connect(addr).unwrap();
        let client = Arc::new(TcpClient::new(client_stream));

        // Поток для обработки событий, полученных клиентом
        let client_events = {
            let client = client.clone();
            thread::spawn(move || {
                client.listen(|event| {
                    info!("Клиент получил событие: {:?}", event);
                });
            })
        };

        // Поток для отправки события от клиента к серверу
        let client_to_server = {
            let client = client.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(3));
                let event = Event {
                    opcode: 0,
                    data: "Hello from client".to_string(),
                };
                if let Err(e) = client.send(&event) {
                    error!("Ошибка отправки события от клиента: {:?}", e);
                }
            })
        };

        client_events.join().unwrap();
        client_to_server.join().unwrap();
    });
}
