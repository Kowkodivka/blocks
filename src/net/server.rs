use super::client::Client;
use super::models::Event;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};

pub struct Server {
    listener: TcpListener,
    clients: Arc<Mutex<Vec<Arc<Mutex<Client>>>>>,
    event_sender: Sender<Event>,
    event_receiver: Arc<Mutex<Receiver<Event>>>,
}

impl Server {
    pub async fn new(addr: &str) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        let (event_sender, event_receiver) = channel::<Event>(100);
        let clients = Arc::new(Mutex::new(Vec::new()));

        Ok(Arc::new(Self {
            listener,
            clients,
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }))
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let client = Arc::new(Mutex::new(Client::new(stream)));

            self.register_client(client.clone()).await;

            let sender_clone = self.event_sender.clone();
            tokio::spawn(async move {
                let client = client.lock().await;
                if let Err(e) = client
                    .listen(move |event| {
                        let sender = sender_clone.clone();
                        tokio::spawn(async move {
                            if sender.send(event).await.is_err() {
                                eprintln!("Failed to forward event");
                            }
                        });
                    })
                    .await
                {
                    eprintln!("Error listening to client events: {:?}", e);
                }
            });
        }
    }

    pub async fn listen_events<F>(&self, mut callback: F)
    where
        F: FnMut(Event) + Send + 'static,
    {
        let receiver = self.event_receiver.clone();
        tokio::spawn(async move {
            let mut receiver = receiver.lock().await;
            while let Some(event) = receiver.recv().await {
                callback(event);
            }
        });
    }

    pub async fn broadcast_event(&self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        let clients = self.clients.lock().await;
        for client in clients.iter() {
            let client = client.lock().await;
            if let Err(e) = client.send_event(&event).await {
                eprintln!("Failed to send event to client: {:?}", e);
            }
        }
        Ok(())
    }

    async fn register_client(&self, client: Arc<Mutex<Client>>) {
        let mut clients = self.clients.lock().await;
        clients.push(client);
    }
}
