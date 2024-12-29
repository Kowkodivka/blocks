use super::{client::Client, models::Event};
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{channel, Receiver, Sender},
        RwLock,
    },
};

pub struct Server {
    listener: TcpListener,
    clients: Arc<RwLock<Vec<Arc<Client>>>>,
    event_sender: Sender<Event>,
    event_receiver: Arc<tokio::sync::Mutex<Receiver<Event>>>,
}

impl Server {
    pub async fn new(addr: &str) -> tokio::io::Result<Arc<Self>> {
        let listener = TcpListener::bind(addr).await?;
        let (event_sender, event_receiver) = channel::<Event>(100);

        Ok(Arc::new(Self {
            listener,
            clients: Arc::new(RwLock::new(Vec::new())),
            event_sender,
            event_receiver: Arc::new(tokio::sync::Mutex::new(event_receiver)),
        }))
    }

    pub async fn start(self: Arc<Self>) -> tokio::io::Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let client = Arc::new(Client::new(stream));
            self.register_client(client.clone()).await;

            let server = Arc::clone(&self);
            tokio::spawn(async move {
                if let Err(e) = server.handle_client(client).await {
                    eprintln!("Error handling client: {:?}", e);
                }
            });
        }
    }

    pub async fn broadcast_event(&self, event: Event) {
        let clients = self.clients.read().await;
        for client in clients.iter() {
            let event = event.clone();
            let client = client.clone();
            tokio::spawn(async move {
                if let Err(e) = client.send(&event).await {
                    eprintln!("Failed to send event to client: {:?}", e);
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
            callback(event);
        }
    }

    async fn handle_client(self: Arc<Self>, client: Arc<Client>) -> tokio::io::Result<()> {
        let sender = self.event_sender.clone();
        client
            .listen(move |event| {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let _ = sender.send(event).await;
                });
            })
            .await;
        Ok(())
    }

    async fn register_client(&self, client: Arc<Client>) {
        let mut clients = self.clients.write().await;
        clients.push(client);
    }
}
