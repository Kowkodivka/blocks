use bevy::log::info;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use super::models;
use super::{client::Client, models::Event};

pub struct Server {
    listener: TcpListener,
    clients: Arc<Mutex<Vec<Client>>>,
    sender: Sender<Event>,
}

impl Server {
    async fn new(addr: &str) -> Arc<Self> {
        let listener = TcpListener::bind(addr).await.unwrap();
        let (sender, receiver) = channel::<Event>(100);
        let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

        let server = Arc::new(Self {
            listener,
            clients: clients.clone(),
            sender,
        });

        let server_clone = Arc::clone(&server);
        tokio::spawn(async move {
            server_clone.event_dispatcher(receiver).await;
        });

        server
    }

    async fn start(&self) {
        loop {
            let (stream, _) = self.listener.accept().await.unwrap();
            let client = Client::new(stream);
            self.clients.lock().unwrap().push(client);
        }
    }

    async fn broadcast_event(&self, event: Event) {
        self.sender.send(event).await.unwrap();
    }

    async fn event_dispatcher(&self, mut receiver: Receiver<Event>) {
        while let Some(event) = receiver.recv().await {
            if let Some(event_type) = models::deserialize(&event) {
                info!("{:#?}", event_type);
            }
        }
    }
}
