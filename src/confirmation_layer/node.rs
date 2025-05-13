use std::collections::HashMap;
use tokio::time::{Duration, interval};
use crate::types::{
    BlockId, ChainId, SubBlock, CLTransaction, Transaction,
};
use super::{ConfirmationLayer, ConfirmationLayerError};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A wrapper for Arc<Mutex<ConfirmationLayerNode>> that implements ConfirmationLayer
#[derive(Clone)]
pub struct ConfirmationLayerNodeWrapper {
    pub inner: Arc<Mutex<ConfirmationLayerNode>>,
}

impl ConfirmationLayerNodeWrapper {
    pub fn new(inner: ConfirmationLayerNode) -> Self {
        Self { inner: Arc::new(Mutex::new(inner)) }
    }

    /// Set the sender for transactions from Hyper Scheduler
    pub async fn set_sender_hs_to_cl(&self, sender: mpsc::Sender<CLTransaction>) {
        let mut node = self.inner.lock().await;
        node.sender_hs_to_cl = sender;
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

    /// Get the sender for messages to Hyper IG
    pub async fn get_sender_cl_to_hig(&self) -> mpsc::Sender<SubBlock> {
        let node = self.inner.lock().await;
        node.sender_cl_to_hig.clone()
    }

    /// Get the sender for transactions from Hyper Scheduler
    pub async fn get_sender_hs_to_cl(&self) -> mpsc::Sender<CLTransaction> {
        let node = self.inner.lock().await;
        node.sender_hs_to_cl.clone()
    }

    /// Get the number of processed transactions
    pub async fn get_processed_transactions_count(&self) -> usize {
        let node = self.inner.lock().await;
        node.pending_txs.values().map(|txs| txs.len()).sum()
    }

    /// Get the number of pending transactions
    pub async fn get_pending_transactions_count(&self) -> usize {
        let node = self.inner.lock().await;
        node.pending_txs.values().map(|txs| txs.len()).sum()
    }

    /// Get the number of blocks produced
    pub async fn get_blocks_count(&self) -> usize {
        let node = self.inner.lock().await;
        node.subblocks.len()
    }

    /// Get the current block number
    pub async fn get_current_block_number(&self) -> u64 {
        let node = self.inner.lock().await;
        node.current_block.0.parse::<u64>().unwrap()
    }

    /// Get the registered chains
    pub async fn get_registered_chains(&self) -> Result<Vec<ChainId>, ConfirmationLayerError> {
        let node = self.inner.lock().await;
        Ok(node.chains.clone())
    }
}

#[async_trait::async_trait]
impl ConfirmationLayer for ConfirmationLayerNodeWrapper {
    async fn register_chain(&mut self, chain_id: ChainId) -> Result<BlockId, ConfirmationLayerError> {
        let mut node = self.inner.lock().await;
        node.register_chain(chain_id).await
    }

    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        let mut node = self.inner.lock().await;
        node.submit_transaction(transaction).await
    }

    async fn get_subblock(&self, chain_id: ChainId, block_id: BlockId) -> Result<SubBlock, ConfirmationLayerError> {
        let node = self.inner.lock().await;
        node.get_subblock(chain_id, block_id).await
    }

    async fn get_current_block(&self) -> Result<BlockId, ConfirmationLayerError> {
        let node = self.inner.lock().await;
        node.get_current_block().await
    }

    async fn get_registered_chains(&self) -> Result<Vec<ChainId>, ConfirmationLayerError> {
        let node = self.inner.lock().await;
        node.get_registered_chains().await
    }

    async fn set_block_interval(&mut self, interval: Duration) -> Result<(), ConfirmationLayerError> {
        let mut node = self.inner.lock().await;
        node.set_block_interval(interval).await
    }

    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError> {
        let node = self.inner.lock().await;
        node.get_block_interval().await
    }
}

/// A simple node implementation of the ConfirmationLayer
pub struct ConfirmationLayerNode {
    /// Currently registered chains
    pub chains: Vec<ChainId>,
    /// Current block ID
    pub current_block: BlockId,
    /// Block interval
    pub block_interval: Duration,
    /// Pending transactions for each chain
    pub pending_txs: HashMap<ChainId, Vec<CLTransaction>>,
    /// Stored subblocks by chain and block ID
    pub subblocks: HashMap<(ChainId, BlockId), SubBlock>,
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
            chains: Vec::new(),
            current_block: BlockId("0".to_string()),
            block_interval: Duration::from_millis(100),
            pending_txs: HashMap::new(),
            subblocks: HashMap::new(),
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
            chains: Vec::new(),
            current_block: BlockId("0".to_string()),
            block_interval: interval,
            pending_txs: HashMap::new(),
            subblocks: HashMap::new(),
            receiver_hs_to_cl,
            sender_cl_to_hig,
            sender_hs_to_cl,
        })
    }

    /// Start the message processing loop
    pub async fn start(&mut self) {
        while let Some(transaction) = self.receiver_hs_to_cl.recv().await {
            tracing::info!("Received transaction from HS: {:?}", transaction);
            if let Err(e) = self.submit_transaction(transaction).await {
                tracing::error!("Failed to process transaction: {}", e);
            }
        }
    }

    /// Start the block production loop
    pub async fn start_block_production(&mut self) {
        println!("Starting block production loop");
        let mut interval = interval(self.block_interval.clone());
        
            loop {
                interval.tick().await;
            
            // Process any new transactions from the channel
            while let Ok(transaction) = self.receiver_hs_to_cl.try_recv() {
                println!("[Processor] received transaction from chain {}: {}", transaction.chain_id.0, transaction.data);
                if self.chains.contains(&transaction.chain_id) {
                    let chain_txs = self.pending_txs.entry(transaction.chain_id.clone())
                        .or_insert_with(Vec::new);
                    chain_txs.push(transaction);
                }
            }
            
            // Create new subblocks for each chain
            let mut new_subblocks = Vec::new();
            let current_block = self.current_block.clone();
            
            // Process pending transactions for this block
            for (chain_id, txs) in self.pending_txs.iter_mut() {
                if !txs.is_empty() {
                                let subblock = SubBlock {
                                    chain_id: chain_id.clone(),
                        block_id: current_block.clone(),
                        transactions: txs.drain(..).map(|tx| Transaction {
                            id: tx.id.clone(),
                            data: tx.data.clone(),
                                    }).collect(),
                                };
                    new_subblocks.push((chain_id.clone(), subblock));
                }
            }
            
            // Store subblocks and send them
            for (chain_id, subblock) in new_subblocks {
                self.subblocks.insert((chain_id.clone(), current_block.clone()), subblock.clone());
                
                // Try to send the subblock, but don't break the loop if it fails
                if let Err(e) = self.sender_cl_to_hig.send(subblock).await {
                    println!("Failed to send subblock to HIG: {}", e);
                    // Don't break here, just continue with the next subblock
                    }
                }
                
                // Increment block ID
            let next_block = (current_block.0.parse::<u64>().unwrap() + 1).to_string();
            self.current_block = BlockId(next_block);
            println!("Block production complete, next block: {:?}", self.current_block);
            
            // Give other tasks a chance to run
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

#[async_trait::async_trait]
impl ConfirmationLayer for ConfirmationLayerNode {
    async fn register_chain(&mut self, chain_id: ChainId) -> Result<BlockId, ConfirmationLayerError> {
        if self.chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainAlreadyRegistered(chain_id));
        }
        self.chains.push(chain_id);
        Ok(self.current_block.clone())
    }

    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        if !self.chains.contains(&transaction.chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(transaction.chain_id));
        }
        let chain_txs = self.pending_txs.entry(transaction.chain_id.clone())
            .or_insert_with(Vec::new);
        chain_txs.push(transaction);
        Ok(())
    }

    async fn get_subblock(&self, chain_id: ChainId, block_id: BlockId) -> Result<SubBlock, ConfirmationLayerError> {
        if !self.chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(chain_id));
        }
        self.subblocks.get(&(chain_id.clone(), block_id.clone()))
            .cloned()
            .ok_or_else(|| ConfirmationLayerError::SubBlockNotFound(chain_id, block_id))
    }

    async fn get_current_block(&self) -> Result<BlockId, ConfirmationLayerError> {
        Ok(self.current_block.clone())
    }

    async fn get_registered_chains(&self) -> Result<Vec<ChainId>, ConfirmationLayerError> {
        Ok(self.chains.clone())
    }

    async fn set_block_interval(&mut self, interval: Duration) -> Result<(), ConfirmationLayerError> {
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        self.block_interval = interval;
        Ok(())
    }

    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError> {
        Ok(self.block_interval.clone())
    }
} 