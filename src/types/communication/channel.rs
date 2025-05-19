use tokio::sync::mpsc;
use super::Message;

/// A generic channel for communication between components
pub struct Channel<T> {
    tx: mpsc::Sender<Message<T>>,
    rx: mpsc::Receiver<Message<T>>,
}

impl<T> Channel<T> {
    /// Create a new channel with the specified buffer size
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        Self { tx, rx }
    }

    /// Split the channel into sender and receiver
    pub fn split(self) -> (Sender<T>, Receiver<T>) {
        (
            Sender { tx: self.tx },
            Receiver { rx: self.rx },
        )
    }
}

/// A sender for sending messages
#[derive(Clone)]
pub struct Sender<T> {
    tx: mpsc::Sender<Message<T>>,
}

impl<T> Sender<T> {
    /// Send a message
    pub async fn send(&self, data: T) -> Result<(), mpsc::error::SendError<Message<T>>> {
        self.tx.send(Message::new(data)).await
    }
}

/// A receiver for receiving messages
pub struct Receiver<T> {
    rx: mpsc::Receiver<Message<T>>,
}

impl<T> Receiver<T> {
    /// Receive a message
    pub async fn receive(&mut self) -> Option<Message<T>> {
        self.rx.recv().await
    }
} 