use super::{
    client::TcpClient,
    models::{Event, EventType},
};
use bevy::prelude::*;
use std::{
    any::Any,
    io,
    net::TcpListener,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

pub struct TcpServer {
    listener: TcpListener,
    clients: Arc<Mutex<Vec<Arc<TcpClient>>>>,
    event_sender: Sender<Event>,
    event_receiver: Arc<Mutex<Receiver<Event>>>,
}

impl TcpServer {
    pub fn new(addr: &str) -> io::Result<Arc<Self>> {
        let listener = TcpListener::bind(addr)?;
        let (event_sender, event_receiver) = mpsc::channel::<Event>();

        Ok(Arc::new(Self {
            listener,
            clients: Arc::new(Mutex::new(Vec::new())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }))
    }

    pub fn start(self: Arc<Self>) -> io::Result<()> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client = Arc::new(TcpClient::new(stream));
                    self.register_client(client.clone());

                    let server = Arc::clone(&self);
                    thread::spawn(move || {
                        if let Err(e) = server.handle_client(client) {
                            error!("Error handling client: {:?}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {:?}", e);
                }
            }
        }
        Ok(())
    }

    pub fn broadcast_event(&self, event: Event) {
        let clients = self.clients.lock().unwrap();
        for client in clients.iter() {
            let event = event.clone();
            let client = client.clone();
            thread::spawn(move || {
                if let Err(e) = client.send(&event) {
                    error!("Failed to send event to client: {:?}", e);
                }
            });
        }
    }

    pub fn listen_events<F>(&self, mut callback: F)
    where
        F: FnMut(Event) + Send + 'static,
    {
        let receiver = self.event_receiver.lock().unwrap();
        for event in receiver.iter() {
            callback(event);
        }
    }

    // TODO: убрать кучу match для событий
    pub fn listen_event<E, F>(&self, mut callback: F)
    where
        E: 'static + Clone + Send,
        F: FnMut(E) + Send + 'static,
    {
        let receiver = Arc::clone(&self.event_receiver);

        thread::spawn(move || {
            let receiver = receiver.lock().unwrap();
            for event in receiver.iter() {
                let event_type = EventType::from_event(&event);

                match event_type {
                    EventType::Hello(e) => {
                        if let Some(specific_event) = Self::try_cast::<E>(&e) {
                            callback(specific_event);
                        }
                    }
                    EventType::Heartbeat(e) => {
                        if let Some(specific_event) = Self::try_cast::<E>(&e) {
                            callback(specific_event);
                        }
                    }
                    EventType::Unknown(opcode, data) => {
                        error!("Unknown event received with opcode {}: {}", opcode, data);
                    }
                }
            }
        });
    }

    fn handle_client(self: Arc<Self>, client: Arc<TcpClient>) -> io::Result<()> {
        let sender = self.event_sender.clone();
        client.listen(move |event| {
            let sender = sender.clone();
            thread::spawn(move || {
                let _ = sender.send(event);
            });
        });
        Ok(())
    }

    fn register_client(&self, client: Arc<TcpClient>) {
        let mut clients = self.clients.lock().unwrap();
        clients.push(client);
    }

    fn try_cast<T: 'static + Clone>(obj: &dyn Any) -> Option<T> {
        obj.downcast_ref::<T>().cloned()
    }
}
