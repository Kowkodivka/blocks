use super::client::Client;
use super::models::Event;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};

pub struct Server {
    listener: TcpListener,
    pub clients: Arc<Mutex<Vec<Client>>>,
    event_sender: mpsc::Sender<Event>,
    event_receiver: Arc<Mutex<mpsc::Receiver<Event>>>,
}

impl Server {
    pub async fn new(addr: &str) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        let (event_sender, event_receiver) = mpsc::channel::<Event>(100);

        Ok(Arc::new(Self {
            listener,
            clients: Arc::new(Mutex::new(Vec::new())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }))
    }

    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let client = Client::new(stream);

            println!("New client connected");

            self.register_client(client.clone()).await;

            let server_clone = Arc::clone(&self);
            tokio::spawn(async move {
                if let Err(e) = client
                    .listen(move |event| {
                        let sender = server_clone.event_sender.clone();
                        tokio::spawn(async move {
                            let _ = sender.send(event).await;
                        });
                    })
                    .await
                {
                    eprintln!("Client listener error: {:?}", e);
                }
            });
        }
    }

    pub async fn listen_events<F>(&self, mut callback: F)
    where
        F: FnMut(Event) + Send + 'static,
    {
        let mut receiver = self.event_receiver.lock().await;

        while let Some(event) = receiver.recv().await {
            println!("Event received by server: {:?}", event);
            callback(event);
        }
    }

    pub async fn broadcast_event(&self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        let mut clients = self.clients.lock().await;

        let mut connected_clients = Vec::new();
        for client in clients.iter() {
            if client.is_connected().await {
                connected_clients.push(client.clone());
            } else {
                println!("Removing disconnected client");
            }
        }

        *clients = connected_clients;

        let send_tasks = clients.iter().map(|client| {
            let event = event.clone();
            async move {
                if let Err(e) = client.send_event(&event).await {
                    eprintln!("Failed to send event to client: {:?}", e);
                }
            }
        });

        futures::future::join_all(send_tasks).await;

        Ok(())
    }

    async fn register_client(&self, client: Client) {
        let mut clients = self.clients.lock().await;
        clients.push(client);
        println!("Client registered. Total clients: {}", clients.len());
    }
}
