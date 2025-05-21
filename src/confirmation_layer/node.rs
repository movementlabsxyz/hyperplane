use tokio::time::Duration;
use tokio::sync::mpsc;
use crate::types::{Transaction, ChainId, CLTransaction, SubBlock};
use super::{ConfirmationLayer, ConfirmationLayerError};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// The internal state of the ConfirmationLayerNode
pub struct ConfirmationLayerState {
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
}

/// A simple node implementation of the ConfirmationLayer
pub struct ConfirmationLayerNode {
    /// The internal state of the node
    pub state: Arc<Mutex<ConfirmationLayerState>>,
    /// Receiver for messages from Hyper Scheduler
    receiver_hs_to_cl: Option<mpsc::Receiver<CLTransaction>>,
    /// Sender for messages to Hyper IG
    sender_cl_to_hig: Option<mpsc::Sender<SubBlock>>,
}

impl ConfirmationLayerNode {
    /// Create a new ConfirmationLayerNode with default settings
    pub fn new(receiver_hs_to_cl: mpsc::Receiver<CLTransaction>, sender_cl_to_hig: mpsc::Sender<SubBlock>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ConfirmationLayerState {
                registered_chains: Vec::new(),
                current_block: 0,
                block_interval: Duration::from_millis(100),
                pending_transactions: Vec::new(),
                processed_transactions: Vec::new(),
                blocks: Vec::new(),
                block_transactions: HashMap::new(),
            })),
            receiver_hs_to_cl: Some(receiver_hs_to_cl),
            sender_cl_to_hig: Some(sender_cl_to_hig),
        }
    }

    /// Create a new ConfirmationLayerNode with a custom block interval
    pub fn new_with_block_interval(
        receiver_hs_to_cl: mpsc::Receiver<CLTransaction>,
        sender_cl_to_hig: mpsc::Sender<SubBlock>,
        interval: Duration
    ) -> Result<Self, ConfirmationLayerError> {
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        Ok(Self {
            state: Arc::new(Mutex::new(ConfirmationLayerState {
                registered_chains: Vec::new(),
                current_block: 0,
                block_interval: interval,
                pending_transactions: Vec::new(),
                processed_transactions: Vec::new(),
                blocks: Vec::new(),
                block_transactions: HashMap::new(),
            })),
            receiver_hs_to_cl: Some(receiver_hs_to_cl),
            sender_cl_to_hig: Some(sender_cl_to_hig),
        })
    }

    /// Process messages and create blocks
    pub async fn process_messages_and_create_blocks(node: Arc<Mutex<Self>>) {
        let mut interval = tokio::time::interval(node.lock().await.state.lock().await.block_interval);
        loop {
            interval.tick().await;
            println!("  [BLOCK]   Height: {}", node.lock().await.state.lock().await.current_block);

            // Process any pending transactions
            {
                let mut state = node.lock().await;
                while let Ok(transaction) = state.receiver_hs_to_cl.as_mut().unwrap().try_recv() {
                    println!("  [BLOCK]   received transaction for chains {:?}: {}", 
                        transaction.constituent_chains.iter().map(|c| c.0.clone()).collect::<Vec<_>>(), 
                        transaction.data);
                    let mut inner_state = state.state.lock().await;
                    // Check if all chains are registered
                    if transaction.constituent_chains.iter().all(|c| inner_state.registered_chains.contains(c)) {
                        inner_state.pending_transactions.push(transaction);
                    }
                }
            }

            // Process the current block
            let (current_block, processed_this_block, registered_chains) = {
                let state = node.lock().await;
                let mut inner_state = state.state.lock().await;
                inner_state.current_block += 1;
                let current_block = inner_state.current_block;
                
                // Process pending transactions for this block
                let mut processed_this_block = Vec::new();
                let mut remaining = Vec::new();
                let registered_chains = inner_state.registered_chains.clone();
                for tx in inner_state.pending_transactions.drain(..) {
                    // Check if all chains are registered
                    if tx.constituent_chains.iter().all(|c| registered_chains.contains(c)) {
                        // Add to processed transactions for each chain
                        for chain_id in &tx.constituent_chains {
                            processed_this_block.push((chain_id.clone(), tx.clone()));
                        }
                    } else {
                        remaining.push(tx);
                    }
                }
                inner_state.pending_transactions = remaining;
                
                // Create a block
                inner_state.blocks.push(current_block);
                
                // Store transactions for this block
                inner_state.block_transactions.insert(current_block, processed_this_block.clone());
                
                // Add processed transactions
                inner_state.processed_transactions.extend(processed_this_block.clone());
                
                (current_block, processed_this_block, registered_chains)
            };

            // Send subblocks to each chain
            {
                #[allow(unused_mut)]
                let mut state = node.lock().await;
                for chain_id in &registered_chains {
                    let subblock = SubBlock {
                        chain_id: chain_id.clone(),
                        block_id: current_block,
                        transactions: processed_this_block
                            .iter()
                            .filter(|(cid, _)| cid == chain_id)
                            .map(|(_, tx)| Transaction {
                                id: tx.id.clone(),
                                this_chain_id: chain_id.clone(),
                                data: tx.data.clone(),
                                constituent_chains: tx.constituent_chains.clone(),
                            })
                            .collect(),
                    };
                    if let Err(e) = state.sender_cl_to_hig.as_mut().unwrap().send(subblock).await {
                        println!("  [BLOCK]   Error sending subblock: {}", e);
                        continue;
                    }
                }
            }
        }
    }

    /// Start the message processing and block production loop
    pub async fn start(node: Arc<Mutex<Self>>) {
        println!("  [CL]   Starting block production");
        tokio::spawn(async move { Self::process_messages_and_create_blocks(node).await });
    }
}

#[async_trait::async_trait]
impl ConfirmationLayer for ConfirmationLayerNode {
    async fn register_chain(&mut self, chain_id: ChainId) -> Result<u64, ConfirmationLayerError> {
        let mut state = self.state.lock().await;
        if state.registered_chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainAlreadyRegistered(chain_id));
        }
        state.registered_chains.push(chain_id);
        Ok(state.current_block)
    }

    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        let mut state = self.state.lock().await;
        
        // Check if all chains are registered
        for chain_id in &transaction.constituent_chains {
            if !state.registered_chains.contains(chain_id) {
                return Err(ConfirmationLayerError::ChainNotFound(chain_id.clone()));
            }
        }
        
        state.pending_transactions.push(transaction);
        Ok(())
    }

    async fn get_subblock(&self, chain_id: ChainId, block_id: u64) -> Result<SubBlock, ConfirmationLayerError> {
        let state = self.state.lock().await;
        if !state.registered_chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(chain_id));
        }

        // Get transactions for this block, or return empty list if no transactions
        let transactions = state.block_transactions
            .get(&block_id)
            .map(|txs| txs.iter()
                .filter(|(cid, _)| cid == &chain_id)
                .map(|(_, tx)| Transaction {
                    id: tx.id.clone(),
                    this_chain_id: chain_id.clone(),
                    data: tx.data.clone(),
                    constituent_chains: tx.constituent_chains.clone(),
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
        let state = self.state.lock().await;
        Ok(state.current_block)
    }

    async fn get_registered_chains(&self) -> Result<Vec<ChainId>, ConfirmationLayerError> {
        let state = self.state.lock().await;
        Ok(state.registered_chains.clone())
    }

    async fn set_block_interval(&mut self, interval: Duration) -> Result<(), ConfirmationLayerError> {
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        let mut state = self.state.lock().await;
        state.block_interval = interval;
        Ok(())
    }

    async fn get_block_interval(&self) -> Result<Duration, ConfirmationLayerError> {
        let state = self.state.lock().await;
        Ok(state.block_interval)
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