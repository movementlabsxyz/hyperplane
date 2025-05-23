use async_trait::async_trait;
use std::time::Duration;
use thiserror::Error;
use crate::types::{ChainId, SubBlock, CLTransaction};
use tokio::sync::mpsc; // Import the correct mpsc module

pub mod node;
pub use node::ConfirmationLayerNode;

#[cfg(test)]
mod tests;

#[derive(Debug, Error)]
pub enum ConfirmationLayerError {
    #[error("Chain not found: {0}")]
    ChainNotFound(ChainId),
    #[error("Chain already registered: {0}")]
    ChainAlreadyRegistered(ChainId),
    #[error("Invalid block interval: {0:?}")]
    InvalidBlockInterval(Duration),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Communication error: {0}")]
    Communication(String),
    #[error("SubBlock not found for chain {0} and block {1}")]
    SubBlockNotFound(ChainId, u64),
}

#[async_trait]
pub trait ConfirmationLayer: Send + Sync {
    /// Register a new chain with the confirmation layer
    async fn register_chain(&mut self, chain_id: ChainId, sender: mpsc::Sender<SubBlock>) -> Result<u64, ConfirmationLayerError>;

    /// Get the current block ID
    async fn get_current_block(&self) -> Result<u64, ConfirmationLayerError>;

    /// Get the subBlock for a specific chain and block
    async fn get_subblock(&self, chain_id: ChainId, block_id: u64) -> Result<SubBlock, ConfirmationLayerError>;

    /// Get all registered chains
    async fn get_registered_chains(&self) -> Result<Vec<ChainId>, ConfirmationLayerError>;

    /// Set the time between blocks
    async fn set_block_interval(&mut self, duration: Duration) -> Result<(), ConfirmationLayerError>;

    /// Get the current block interval
    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError>;

    /// Submit a subblock transaction to be included in the next block
    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError>;
}