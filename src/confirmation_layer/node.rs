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
    pub async fn start_block_production(node: Arc<Mutex<Self>>) {
        let mut interval = tokio::time::interval(node.lock().await.block_interval);
        loop {
            interval.tick().await;
            
            // Process any new transactions from the channel
            {
                let mut state = node.lock().await;
                while let Ok(transaction) = state.receiver_hs_to_cl.try_recv() {
                    println!("[Processor] received transaction for chain {}: {}", transaction.chain_id.0, transaction.data);
                    if state.registered_chains.contains(&transaction.chain_id) {
                        state.pending_transactions.push(transaction);
                    }
                }
            }
            
            // Process block and transactions
            let (current_block, processed_this_block, registered_chains) = {
                let mut state = node.lock().await;
                state.current_block += 1;
                let current_block = state.current_block;
                
                // Process pending transactions for this block
                let mut processed_this_block = Vec::new();
                let mut remaining = Vec::new();
                let registered_chains = state.registered_chains.clone();
                for tx in state.pending_transactions.drain(..) {
                    if registered_chains.contains(&tx.chain_id) {
                        processed_this_block.push((tx.chain_id.clone(), tx.clone()));
                    } else {
                        remaining.push(tx);
                    }
                }
                state.pending_transactions = remaining;
                
                // Create a block
                state.blocks.push(current_block);
                
                // Store transactions for this block
                state.block_transactions.insert(current_block, processed_this_block.clone());
                
                // Add processed transactions
                state.processed_transactions.extend(processed_this_block.clone());
                
                (current_block, processed_this_block, registered_chains)
            };
            
            // Print block status
            if !processed_this_block.is_empty() {
                print!("[Processor] produced block {} with {} transactions", current_block, processed_this_block.len());
                for (_, tx) in &processed_this_block {
                    print!("  - id={}, data={}", tx.id.0, tx.data);
                }
                println!();
            } else {
                println!("[Processor] produced empty block {}", current_block);
            }
            
            // Send subblocks for each chain with only this block's transactions
            let state = node.lock().await;
            for chain_id in &registered_chains {
                let subblock = SubBlock {
                    chain_id: chain_id.clone(),
                    block_id: current_block,
                    transactions: processed_this_block
                        .iter()
                        .filter(|(cid, _)| cid == chain_id)
                        .map(|(_, tx)| Transaction {
                            id: tx.id.clone(),
                            data: tx.data.clone(),
                        })
                        .collect(),
                };
                if let Err(e) = state.sender_cl_to_hig.send(subblock).await {
                    println!("[Processor] Error sending subblock: {}", e);
                    continue;
                }
            }
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
        ConfirmationLayerNode::start_block_production(self.inner.clone()).await;
    }
}

/// Trait for starting node operations
#[async_trait::async_trait]
pub trait NodeStarter {
    /// Start the message processing loop
    async fn start(&mut self);
    /// Start block production
    async fn start_block_production(&self);
}

#[async_trait::async_trait]
impl NodeStarter for Arc<Mutex<ConfirmationLayerNode>> {
    async fn start(&mut self) {
        let mut node = self.lock().await;
        node.start().await;
    }

    async fn start_block_production(&self) {
        ConfirmationLayerNode::start_block_production(self.clone()).await;
    }
}

#[async_trait::async_trait]
impl ConfirmationLayer for ConfirmationLayerNode {
    async fn register_chain(&mut self, chain_id: ChainId) -> Result<u64, ConfirmationLayerError> {
        if self.registered_chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainAlreadyRegistered(chain_id));
        }
        self.registered_chains.push(chain_id);
        Ok(self.current_block)
    }

    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        if !self.registered_chains.contains(&transaction.chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(transaction.chain_id));
        }
        self.pending_transactions.push(transaction);
        Ok(())
    }

    async fn get_subblock(&self, chain_id: ChainId, block_id: u64) -> Result<SubBlock, ConfirmationLayerError> {
        if !self.registered_chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(chain_id));
        }

        // Get transactions for this block, or return empty list if no transactions
        let transactions = self.block_transactions
            .get(&block_id)
            .map(|txs| txs.iter()
                .filter(|(cid, _)| cid == &chain_id)
                .map(|(_, tx)| Transaction {
                    id: tx.id.clone(),
                    data: tx.data.clone(),
                })
                .collect())
            .unwrap_or_default();

        Ok(SubBlock {
            chain_id: chain_id.clone(),
            block_id,
            transactions,
        })
    }

    async fn get_current_block(&self) -> Result<u64, ConfirmationLayerError> {
        Ok(self.current_block)
    }

    async fn get_registered_chains(&self) -> Result<Vec<ChainId>, ConfirmationLayerError> {
        Ok(self.registered_chains.clone())
    }

    async fn set_block_interval(&mut self, interval: Duration) -> Result<(), ConfirmationLayerError> {
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        self.block_interval = interval;
        Ok(())
    }

    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError> {
        Ok(self.block_interval)
    }
}

#[async_trait::async_trait]
impl ConfirmationLayer for Arc<Mutex<ConfirmationLayerNode>> {
    async fn register_chain(&mut self, chain_id: ChainId) -> Result<u64, ConfirmationLayerError> {
        let mut node = self.lock().await;
        node.register_chain(chain_id).await
    }

    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        let mut node = self.lock().await;
        node.submit_transaction(transaction).await
    }

    async fn get_subblock(&self, chain_id: ChainId, block_id: u64) -> Result<SubBlock, ConfirmationLayerError> {
        let node = self.lock().await;
        node.get_subblock(chain_id, block_id).await
    }

    async fn get_current_block(&self) -> Result<u64, ConfirmationLayerError> {
        let node = self.lock().await;
        node.get_current_block().await
    }

    async fn get_registered_chains(&self) -> Result<Vec<ChainId>, ConfirmationLayerError> {
        let node = self.lock().await;
        node.get_registered_chains().await
    }

    async fn set_block_interval(&mut self, interval: Duration) -> Result<(), ConfirmationLayerError> {
        let mut node = self.lock().await;
        node.set_block_interval(interval).await
    }

    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError> {
        let node = self.lock().await;
        node.get_block_interval().await
    }
} 