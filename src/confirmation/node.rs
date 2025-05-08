use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use crate::types::{BlockId, ChainId, ChainRegistration, SubBlock};
use super::{ConfirmationLayer, ConfirmationLayerError};

/// A simple node implementation of the ConfirmationLayer
pub struct ConfirmationNode {
    /// Currently registered chains
    chains: Arc<RwLock<HashMap<ChainId, ChainRegistration>>>,
    /// Current block ID
    current_block: Arc<RwLock<BlockId>>,
    /// Block interval
    block_interval: Arc<RwLock<Duration>>,
}

impl ConfirmationNode {
    /// Create a new confirmation node with default settings
    pub fn new() -> Self {
        Self {
            chains: Arc::new(RwLock::new(HashMap::new())),
            current_block: Arc::new(RwLock::new(BlockId(0))),
            block_interval: Arc::new(RwLock::new(Duration::from_secs(1))), // Default 1 second
        }
    }

    /// Create a new confirmation node with custom block interval
    pub fn with_block_interval(interval: Duration) -> Result<Self, ConfirmationLayerError> {
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        Ok(Self {
            chains: Arc::new(RwLock::new(HashMap::new())),
            current_block: Arc::new(RwLock::new(BlockId(0))),
            block_interval: Arc::new(RwLock::new(interval)),
        })
    }
}

#[async_trait::async_trait]
impl ConfirmationLayer for ConfirmationNode {
    async fn register_chain(&mut self, chain_id: ChainId) -> Result<BlockId, ConfirmationLayerError> {
        let mut chains = self.chains.write().await;
        
        // Check if chain is already registered
        if chains.contains_key(&chain_id) {
            return Err(ConfirmationLayerError::ChainAlreadyRegistered(chain_id));
        }

        // Get current block for registration
        let current_block = self.current_block.read().await.clone();
        
        // Register the chain
        chains.insert(
            chain_id.clone(),
            ChainRegistration {
                chain_id,
                registration_block: current_block.clone(),
                active: true,
            },
        );

        Ok(current_block)
    }

    async fn get_current_block(&self) -> Result<BlockId, ConfirmationLayerError> {
        Ok(self.current_block.read().await.clone())
    }

    async fn get_subblock(&self, chain_id: ChainId, block_id: BlockId) -> Result<SubBlock, ConfirmationLayerError> {
        let chains = self.chains.read().await;
        
        // Check if chain exists and is active
        let registration = chains.get(&chain_id)
            .ok_or_else(|| ConfirmationLayerError::ChainNotFound(chain_id.clone()))?;
        
        if !registration.active {
            return Err(ConfirmationLayerError::ChainNotFound(chain_id));
        }

        // For now, return empty subblock
        Ok(SubBlock {
            block_id,
            chain_id,
            transactions: Vec::new(),
        })
    }

    async fn get_registered_chains(&self) -> Result<Vec<ChainRegistration>, ConfirmationLayerError> {
        Ok(self.chains.read().await.values().cloned().collect())
    }

    async fn set_block_interval(&mut self, duration: Duration) -> Result<(), ConfirmationLayerError> {
        if duration.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(duration));
        }
        *self.block_interval.write().await = duration;
        Ok(())
    }

    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError> {
        Ok(*self.block_interval.read().await)
    }
} 