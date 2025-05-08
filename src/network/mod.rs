use async_trait::async_trait;
use thiserror::Error;
use libp2p::PeerId;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait Network: Send + Sync {
    async fn connect(&mut self, peer_id: PeerId) -> Result<(), NetworkError>;
    async fn disconnect(&mut self, peer_id: PeerId) -> Result<(), NetworkError>;
    async fn send(&mut self, peer_id: PeerId, data: &[u8]) -> Result<(), NetworkError>;
    async fn broadcast(&mut self, data: &[u8]) -> Result<(), NetworkError>;
} 