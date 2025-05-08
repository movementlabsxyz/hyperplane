use async_trait::async_trait;
use std::time::Duration;
use thiserror::Error;
use crate::types::{BlockId, ChainId, ChainRegistration, SubBlock};

mod node;
pub use node::ConfirmationNode;

#[derive(Debug, Error)]
pub enum ConfirmationLayerError {
    #[error("Chain not found: {0}")]
    ChainNotFound(ChainId),
    #[error("Chain already registered: {0}")]
    ChainAlreadyRegistered(ChainId),
    #[error("Invalid block interval: {:?}", .0)]
    InvalidBlockInterval(Duration),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait ConfirmationLayer: Send + Sync {
    /// Register a new chain with the confirmation layer
    async fn register_chain(&mut self, chain_id: ChainId) -> Result<BlockId, ConfirmationLayerError>;

    /// Get the current block ID
    async fn get_current_block(&self) -> Result<BlockId, ConfirmationLayerError>;

    /// Get the subBlock for a specific chain and block
    async fn get_subblock(&self, chain_id: ChainId, block_id: BlockId) -> Result<SubBlock, ConfirmationLayerError>;

    /// Get all registered chains
    async fn get_registered_chains(&self) -> Result<Vec<ChainRegistration>, ConfirmationLayerError>;

    /// Set the time between blocks
    async fn set_block_interval(&mut self, duration: Duration) -> Result<(), ConfirmationLayerError>;

    /// Get the current block interval
    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError>;
} 