use std::fmt::Debug;

/// A generic message type that can be used for any communication
#[derive(Debug)]
pub enum Message<T> {
    /// A message containing data of type T
    Message(T),
}

impl<T> Message<T> {
    /// Create a new message
    pub fn new(data: T) -> Self {
        Message::Message(data)
    }
} 