use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use crate::types::{
    BlockId, ChainId, ChainRegistration, SubBlock, CLTransaction, Transaction,
};
use super::{ConfirmationLayer, ConfirmationLayerError};

/// A simple node implementation of the ConfirmationLayer
pub struct ConfirmationNode {
    /// Currently registered chains
    chains: Arc<RwLock<HashMap<ChainId, ChainRegistration>>>,
    /// Current block ID
    current_block: Arc<RwLock<BlockId>>,
    /// Block interval
    block_interval: Arc<RwLock<Duration>>,
    /// Pending transactions for each chain
    pending_txs: Arc<RwLock<HashMap<ChainId, Vec<CLTransaction>>>>,
    /// Stored subblocks by chain and block ID
    subblocks: Arc<RwLock<HashMap<(ChainId, BlockId), SubBlock>>>,
}

impl ConfirmationNode {
    /// Create a new confirmation node with default settings
    pub fn new() -> Self {
        let node = Self {
            chains: Arc::new(RwLock::new(HashMap::new())),
            current_block: Arc::new(RwLock::new(BlockId("0".to_string()))),
            block_interval: Arc::new(RwLock::new(Duration::from_secs(1))),
            pending_txs: Arc::new(RwLock::new(HashMap::new())),
            subblocks: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Start block production
        node.start_block_production();
        node
    }

    /// Create a new confirmation node with custom block interval
    pub fn with_block_interval(interval: Duration) -> Result<Self, ConfirmationLayerError> {
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        let node = Self {
            chains: Arc::new(RwLock::new(HashMap::new())),
            current_block: Arc::new(RwLock::new(BlockId("0".to_string()))),
            block_interval: Arc::new(RwLock::new(interval)),
            pending_txs: Arc::new(RwLock::new(HashMap::new())),
            subblocks: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Start block production
        node.start_block_production();
        Ok(node)
    }

    /// Start the block production loop
    fn start_block_production(&self) {
        let chains = self.chains.clone();
        let current_block = self.current_block.clone();
        let block_interval = self.block_interval.clone();
        let pending_txs = self.pending_txs.clone();
        let subblocks = self.subblocks.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100)); // Check every 100ms
            loop {
                interval.tick().await;
                let _interval_duration = *block_interval.read().await;
                
                // Get current block
                let mut block = current_block.write().await;
                let block_id = block.0.clone();
                println!("Producing block {}", block_id);
                
                // Create subblocks for each chain
                let mut txs = pending_txs.write().await;
                let registered_chains = chains.read().await;
                
                for (chain_id, registration) in registered_chains.iter() {
                    if registration.active {
                        // Take transactions for this chain
                        if let Some(chain_txs) = txs.get_mut(chain_id) {
                            if !chain_txs.is_empty() {
                                println!("Creating subblock for chain {} with {} transactions", chain_id.0, chain_txs.len());
                                // Create a subblock for this chain
                                let subblock = SubBlock {
                                    block_id: BlockId(block_id.clone()),
                                    chain_id: chain_id.clone(),
                                    transactions: chain_txs.drain(..).map(|tx| Transaction {
                                        id: tx.id,
                                        data: tx.data,
                                    }).collect(),
                                };
                                // Store the subblock
                                subblocks.write().await.insert((chain_id.clone(), BlockId(block_id.clone())), subblock);
                            }
                        }
                    }
                }
                
                // Increment block ID
                block.0 = (block.0.parse::<u64>().unwrap() + 1).to_string();
            }
        });
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
        let registration = ChainRegistration {
            chain_id: chain_id.clone(),
            name: format!("Chain {}", chain_id),
            rpc_url: format!("http://localhost:8000"),
            registration_block: current_block.clone(),
            active: true,
        };
        chains.insert(chain_id.clone(), registration);

        // Initialize empty transaction queue for this chain
        self.pending_txs.write().await.insert(chain_id, Vec::new());

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

        // Get stored subblock or return empty one
        let subblocks = self.subblocks.read().await;
        let block_id_clone = block_id.clone();
        Ok(subblocks.get(&(chain_id.clone(), block_id))
            .cloned()
            .unwrap_or_else(|| SubBlock {
                block_id: block_id_clone,
                chain_id,
                transactions: Vec::new(),
            }))
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

    async fn submit_subblock_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        let chains = self.chains.read().await;
        
        // Check if chain exists and is active
        let registration = chains.get(&transaction.chain_id)
            .ok_or_else(|| ConfirmationLayerError::ChainNotFound(transaction.chain_id.clone()))?;
        
        if !registration.active {
            return Err(ConfirmationLayerError::ChainNotFound(transaction.chain_id));
        }

        // Add transaction to pending queue
        let mut pending_txs = self.pending_txs.write().await;
        if let Some(chain_txs) = pending_txs.get_mut(&transaction.chain_id) {
            chain_txs.push(transaction);
            Ok(())
        } else {
            Err(ConfirmationLayerError::Internal("Chain not found in pending transactions".to_string()))
        }
    }
} 