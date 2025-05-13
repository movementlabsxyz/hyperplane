use tokio::time::Duration;
use tokio::sync::mpsc;
use crate::types::{Transaction, ChainId, CLTransaction, SubBlock};
use super::{ConfirmationLayer, ConfirmationLayerError};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// A simple node implementation of the ConfirmationLayer
pub struct ConfirmationLayerNode {
    /// Currently registered chains
    pub registered_chains: Vec<ChainId>,
    /// Current block number
    pub current_block: u64,
    /// Block interval
    pub block_interval: Duration,
    /// Pending transactions
    pub pending_transactions: Vec<CLTransaction>,
    /// Processed transactions
    pub processed_transactions: Vec<(ChainId, CLTransaction)>,
    /// Block history
    pub blocks: Vec<u64>,
    /// Block to transactions mapping
    pub block_transactions: HashMap<u64, Vec<(ChainId, CLTransaction)>>,
    /// Receiver for messages from Hyper Scheduler
    pub receiver_hs_to_cl: mpsc::Receiver<CLTransaction>,
    /// Sender for messages to Hyper IG
    pub sender_cl_to_hig: mpsc::Sender<SubBlock>,
    /// Sender for transactions from Hyper Scheduler
    pub sender_hs_to_cl: mpsc::Sender<CLTransaction>,
}

impl ConfirmationLayerNode {
    /// Create a new ConfirmationLayerNode with default settings
    pub fn new(receiver_hs_to_cl: mpsc::Receiver<CLTransaction>, sender_cl_to_hig: mpsc::Sender<SubBlock>) -> Self {
        let (sender_hs_to_cl, _) = mpsc::channel(100);
        Self {
            registered_chains: Vec::new(),
            current_block: 0,
            block_interval: Duration::from_millis(100),
            pending_transactions: Vec::new(),
            processed_transactions: Vec::new(),
            blocks: Vec::new(),
            block_transactions: HashMap::new(),
            receiver_hs_to_cl,
            sender_cl_to_hig,
            sender_hs_to_cl,
        }
    }

    /// Create a new ConfirmationLayerNode with a custom block interval
    pub fn new_with_block_interval(
        receiver_hs_to_cl: mpsc::Receiver<CLTransaction>,
        sender_cl_to_hig: mpsc::Sender<SubBlock>,
        interval: Duration
    ) -> Result<Self, ConfirmationLayerError> {
        let (sender_hs_to_cl, _) = mpsc::channel(100);
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        Ok(Self {
            registered_chains: Vec::new(),
            current_block: 0,
            block_interval: interval,
            pending_transactions: Vec::new(),
            processed_transactions: Vec::new(),
            blocks: Vec::new(),
            block_transactions: HashMap::new(),
            receiver_hs_to_cl,
            sender_cl_to_hig,
            sender_hs_to_cl,
        })
    }

    /// Start the message processing loop
    pub async fn start(&mut self) {
        while let Some(transaction) = self.receiver_hs_to_cl.recv().await {
            tracing::info!("Received transaction from HS: {:?}", transaction);
            if !self.registered_chains.contains(&transaction.chain_id) {
                tracing::error!("Chain {} not found", transaction.chain_id.0);
                continue;
            }
            self.pending_transactions.push(transaction);
        }
    }

    /// Start the block production loop
    pub async fn start_block_production(&mut self) {
        let mut interval = tokio::time::interval(self.block_interval);
        loop {
            interval.tick().await;
            self.current_block += 1;
            self.blocks.push(self.current_block);
            
            // Process pending transactions
            let mut processed_this_block = Vec::new();
            let mut remaining = Vec::new();
            for tx in self.pending_transactions.drain(..) {
                if self.registered_chains.contains(&tx.chain_id) {
                    processed_this_block.push((tx.chain_id.clone(), tx.clone()));
                } else {
                    remaining.push(tx);
                }
            }
            self.pending_transactions = remaining;

            // Store transactions for this block
            self.block_transactions.insert(self.current_block, processed_this_block.clone());

            // Create and send subblocks for each chain
            for chain_id in &self.registered_chains {
                let subblock = SubBlock {
                    chain_id: chain_id.clone(),
                    block_id: self.current_block,
                    transactions: processed_this_block
                        .iter()
                        .filter(|(cid, _)| cid == chain_id)
                        .map(|(_, tx)| Transaction {
                            id: tx.id.clone(),
                            data: tx.data.clone(),
                        })
                        .collect(),
                };
                if let Err(e) = self.sender_cl_to_hig.send(subblock).await {
                    tracing::error!("Error sending subblock: {}", e);
                    break;
                }
            }
            
            // Update processed transactions
            self.processed_transactions.extend(processed_this_block);
        }
    }
}

/// A wrapper for Arc<Mutex<ConfirmationLayerNode>> that implements ConfirmationLayer
#[derive(Clone)]
pub struct ConfirmationLayerNodeWrapper {
    pub inner: Arc<Mutex<ConfirmationLayerNode>>,
}

impl ConfirmationLayerNodeWrapper {
    pub fn new(inner: ConfirmationLayerNode) -> Self {
        Self { inner: Arc::new(Mutex::new(inner)) }
    }

    /// Start the message processing loop
    pub async fn start(&mut self) {
        let mut node = self.inner.lock().await;
        node.start().await;
    }

    /// Start block production
    pub async fn start_block_production(&self) {
        let mut node = self.inner.lock().await;
        node.start_block_production().await;
    }
}

#[async_trait::async_trait]
impl ConfirmationLayer for ConfirmationLayerNodeWrapper {
    async fn register_chain(&mut self, chain_id: ChainId) -> Result<u64, ConfirmationLayerError> {
        let mut node = self.inner.lock().await;
        if node.registered_chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainAlreadyRegistered(chain_id));
        }
        node.registered_chains.push(chain_id);
        Ok(node.current_block)
    }

    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        let mut node = self.inner.lock().await;
        if !node.registered_chains.contains(&transaction.chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(transaction.chain_id));
        }
        node.pending_transactions.push(transaction);
        Ok(())
    }

    async fn get_subblock(&self, chain_id: ChainId, block_id: u64) -> Result<SubBlock, ConfirmationLayerError> {
        let node = self.inner.lock().await;
        if !node.registered_chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(chain_id));
        }
        
        // Get transactions for this block
        let transactions = node.block_transactions
            .get(&block_id)
            .ok_or_else(|| ConfirmationLayerError::SubBlockNotFound(chain_id.clone(), block_id))?
            .iter()
            .filter(|(cid, _)| cid == &chain_id)
            .map(|(_, tx)| Transaction {
                id: tx.id.clone(),
                data: tx.data.clone(),
            })
            .collect();

        Ok(SubBlock {
            chain_id: chain_id.clone(),
            block_id,
            transactions,
        })
    }

    async fn get_current_block(&self) -> Result<u64, ConfirmationLayerError> {
        let node = self.inner.lock().await;
        Ok(node.current_block)
    }

    async fn get_registered_chains(&self) -> Result<Vec<ChainId>, ConfirmationLayerError> {
        let node = self.inner.lock().await;
        Ok(node.registered_chains.clone())
    }

    async fn set_block_interval(&mut self, interval: Duration) -> Result<(), ConfirmationLayerError> {
        let mut node = self.inner.lock().await;
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        node.block_interval = interval;
        Ok(())
    }

    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError> {
        let node = self.inner.lock().await;
        Ok(node.block_interval)
    }
} 