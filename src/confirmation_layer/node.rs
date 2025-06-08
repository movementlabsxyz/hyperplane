use tokio::time::Duration;
use tokio::sync::mpsc;
use crate::types::{Transaction, ChainId, CLTransaction, SubBlock, CLTransactionId};
use super::{ConfirmationLayer, ConfirmationLayerError};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::{HashMap, HashSet};
use crate::utils::logging::log;

/// The internal state of the ConfirmationLayerNode
pub struct ConfirmationLayerState {
    /// Currently registered chains
    pub registered_chains: Vec<ChainId>,
    /// Current block number
    pub current_block_height: u64,
    /// Block interval
    pub block_interval: Duration,
    /// Pending transactions
    pub pending_transactions: Vec<CLTransaction>,
    /// Processed CL transactions
    pub processed_cltransactions: Vec<CLTransaction>,
    /// Set of processed CL transaction IDs
    pub processed_cltransaction_ids: HashSet<CLTransactionId>,
    /// Processed individual transactions
    pub processed_transactions: Vec<(ChainId, Transaction)>,
    /// Block history
    pub blocks: Vec<u64>,
    /// Block to CL transactions mapping
    pub blocks_cltransactions: HashMap<u64, Vec<CLTransaction>>,
    /// Block to individual transactions mapping
    pub blocks_transactions: HashMap<u64, Vec<(ChainId, Transaction)>>,
    /// Subblock to individual transactions mapping
    pub subblocks_transactions: HashMap<(ChainId, u64), Vec<Transaction>>,
}

/// A simple node implementation of the ConfirmationLayer
pub struct ConfirmationLayerNode {
    /// The internal state of the node
    pub state: Arc<Mutex<ConfirmationLayerState>>,
    /// Receiver for messages from Hyper Scheduler
    receiver_hs_to_cl: Option<mpsc::Receiver<CLTransaction>>,
    /// Replace individual senders with a collection of senders
    pub senders_cl_to_hig: HashMap<String, mpsc::Sender<SubBlock>>, // Map chain ID to its channel
}

impl ConfirmationLayerNode {
    /// Create a new ConfirmationLayerNode with default settings
    pub fn new(receiver_hs_to_cl: mpsc::Receiver<CLTransaction>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ConfirmationLayerState {
                registered_chains: Vec::new(),
                current_block_height: 0,
                block_interval: Duration::from_millis(100),
                pending_transactions: Vec::new(),
                processed_cltransactions: Vec::new(),
                processed_cltransaction_ids: HashSet::new(),
                processed_transactions: Vec::new(),
                blocks: Vec::new(),
                blocks_cltransactions: HashMap::new(),
                blocks_transactions: HashMap::new(),
                subblocks_transactions: HashMap::new(),
            })),
            receiver_hs_to_cl: Some(receiver_hs_to_cl),
            senders_cl_to_hig: HashMap::new(), // Initialize empty map for dynamic channels
        }
    }

    /// Create a new ConfirmationLayerNode with a custom block interval
    pub fn new_with_block_interval(
        receiver_hs_to_cl: mpsc::Receiver<CLTransaction>,
        interval: Duration
    ) -> Result<Self, ConfirmationLayerError> {
        if interval.is_zero() {
            return Err(ConfirmationLayerError::InvalidBlockInterval(interval));
        }
        Ok(Self {
            state: Arc::new(Mutex::new(ConfirmationLayerState {
                registered_chains: Vec::new(),
                current_block_height: 0,
                block_interval: interval,
                pending_transactions: Vec::new(),
                processed_cltransactions: Vec::new(),
                processed_cltransaction_ids: HashSet::new(),
                processed_transactions: Vec::new(),
                blocks: Vec::new(),
                blocks_cltransactions: HashMap::new(),
                blocks_transactions: HashMap::new(),
                subblocks_transactions: HashMap::new(),
            })),
            receiver_hs_to_cl: Some(receiver_hs_to_cl),
            senders_cl_to_hig: HashMap::new(), // Initialize empty map for dynamic channels
        })
    }

    /// Register a new chain
    pub async fn register_chain(&mut self, chain_id: ChainId, sender: mpsc::Sender<SubBlock>) -> Result<u64, ConfirmationLayerError> {
        let mut state = self.state.lock().await;

        if self.senders_cl_to_hig.contains_key(&chain_id.0) {
            log("CL", &format!("Chain {} is already registered.", chain_id.0));
            return Err(ConfirmationLayerError::ChainAlreadyRegistered(chain_id));
        }

        self.senders_cl_to_hig.insert(chain_id.0.clone(), sender);
        log("CL", &format!("Channel registered successfully for chain '{}'.", chain_id.0));

        if !state.registered_chains.contains(&chain_id) {
            state.registered_chains.push(chain_id.clone());
            log("CL", &format!("Chain '{}' added to registered_chains.", chain_id.0));
        }

        Ok(state.current_block_height)
    }

    /// Process messages and create blocks
    pub async fn process_messages_and_create_blocks(node: Arc<Mutex<Self>>) {
        let mut interval = tokio::time::interval(node.lock().await.state.lock().await.block_interval);
        loop {
            interval.tick().await;
            log("BLOCK", &format!("Height: {}", node.lock().await.state.lock().await.current_block_height));

            // Process any pending transactions
            {
                let mut state = node.lock().await;
                while let Ok(transaction) = state.receiver_hs_to_cl.as_mut().unwrap().try_recv() {
                    log("BLOCK", &format!("received transaction for chains {:?}: {}", 
                        transaction.constituent_chains.iter().map(|c| c.0.clone()).collect::<Vec<_>>(), 
                        transaction.transactions.iter().map(|tx| tx.data.clone()).collect::<Vec<_>>().join(", ")));
                    let mut inner_state = state.state.lock().await;
                    // Check if all chains are registered and transaction hasn't been processed
                    let registered_chains = inner_state.registered_chains.clone();
                    let processed_ids = inner_state.processed_cltransaction_ids.clone();
                    let is_valid = transaction.constituent_chains.iter().all(|c| registered_chains.contains(c)) 
                        && !processed_ids.contains(&transaction.id);
                    if is_valid {
                        inner_state.pending_transactions.push(transaction);
                    }
                }
            }

            // Process the current block height
            let (current_block_height, processed_this_block, registered_chains) = {
                let state = node.lock().await;
                let mut inner_state = state.state.lock().await;
                inner_state.current_block_height += 1;
                let current_block_height = inner_state.current_block_height;
                
                // Process pending transactions for this block
                let mut processed_this_block = Vec::new();
                let mut remaining = Vec::new();
                let mut processed_cltransactions = Vec::new();
                let registered_chains = inner_state.registered_chains.clone();
                let processed_ids = inner_state.processed_cltransaction_ids.clone();
                let pending_txs = inner_state.pending_transactions.drain(..).collect::<Vec<_>>();
                
                for cl_tx in pending_txs {
                    // Check if all chains are registered and transaction hasn't been processed
                    let is_valid = cl_tx.constituent_chains.iter().all(|c| registered_chains.contains(c)) 
                        && !processed_ids.contains(&cl_tx.id);
                    if is_valid {
                        // Add to processed transactions for each transaction's this_chain_id
                        for tx in &cl_tx.transactions {
                            processed_this_block.push((tx.target_chain_id.clone(), tx.clone()));
                        }
                        processed_cltransactions.push(cl_tx.clone());
                        inner_state.processed_cltransaction_ids.insert(cl_tx.id.clone());
                    } else {
                        remaining.push(cl_tx);
                    }
                }
                inner_state.pending_transactions = remaining;
                
                // Create a block
                inner_state.blocks.push(current_block_height);
                
                // Store CL transactions for this block
                inner_state.blocks_cltransactions.insert(current_block_height, processed_cltransactions.clone());
                
                // Store individual transactions for this block
                inner_state.blocks_transactions.insert(current_block_height, processed_this_block.clone());
                
                // Add processed transactions
                inner_state.processed_transactions.extend(processed_this_block.clone());
                inner_state.processed_cltransactions.extend(processed_cltransactions);
                
                (current_block_height, processed_this_block, registered_chains)
            };

            // Send subblocks to each registered chain
            {
                let state = node.lock().await;
                for chain_id in &registered_chains {
                    let transactions = processed_this_block
                        .iter()
                        .filter(|(cid, _)| cid == chain_id)
                        .map(|(_, tx)| tx.clone())
                        .collect::<Vec<_>>();

                    let subblock = SubBlock {
                        chain_id: chain_id.clone(),
                        block_height: current_block_height,
                        transactions: transactions.clone(),
                    };

                    // Store transactions for this subblock
                    state.state.lock().await.subblocks_transactions.insert(
                        (chain_id.clone(), current_block_height),
                        transactions
                    );

                    // Send to the registered chain's HIG channel dynamically
                    if let Some(sender) = state.senders_cl_to_hig.get(&chain_id.0) {
                        if let Err(e) = sender.send(subblock).await {
                            log("BLOCK", &format!("Error sending subblock to chain {}: {}", chain_id.0, e));
                        }
                    } else {
                        log("BLOCK", &format!("No channel found for chain {}", chain_id.0));
                        panic!("No channel found for chain {}", chain_id.0);
                    }
                }
            }
        }
    }

    /// Start the message processing and block production loop
    pub async fn start(node: Arc<Mutex<Self>>) {
        log("CL", "Starting block production");
        tokio::spawn(async move { Self::process_messages_and_create_blocks(node).await });
    }
}

#[async_trait::async_trait]
impl ConfirmationLayer for ConfirmationLayerNode {
    async fn submit_transaction(&mut self, transaction: CLTransaction) -> Result<(), ConfirmationLayerError> {
        let mut state = self.state.lock().await;
        
        // Check if all chains are registered and transaction hasn't been processed
        for chain_id in &transaction.constituent_chains {
            if !state.registered_chains.contains(chain_id) {
                return Err(ConfirmationLayerError::ChainNotFound(chain_id.clone()));
            }
        }
        
        if state.processed_cltransaction_ids.contains(&transaction.id) {
            return Err(ConfirmationLayerError::Internal("Transaction already processed".to_string()));
        }
        
        state.pending_transactions.push(transaction);
        Ok(())
    }

    async fn get_subblock(&self, chain_id: ChainId, block_height: u64) -> Result<SubBlock, ConfirmationLayerError> {
        let state = self.state.lock().await;
        if !state.registered_chains.contains(&chain_id) {
            return Err(ConfirmationLayerError::ChainNotFound(chain_id));
        }

        // Get transactions for this block, or return empty list if no transactions
        let transactions = state.blocks_transactions
            .get(&block_height)
            .map(|txs| txs.iter()
                .filter(|(cid, _)| cid == &chain_id)
                .map(|(_, tx)| Transaction::new(
                    tx.id.clone(),
                    tx.target_chain_id.clone(),
                    tx.constituent_chains.clone(),
                    tx.data.clone(),
                ).expect("Failed to create transaction"))
                .collect())
            .unwrap_or_default();

        Ok(SubBlock {
            chain_id: chain_id.clone(),
            block_height: block_height,
            transactions,
        })
    }

    async fn get_current_block(&self) -> Result<u64, ConfirmationLayerError> {
        let state = self.state.lock().await;
        Ok(state.current_block_height)
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

    async fn register_chain(&mut self, chain_id: ChainId, sender: mpsc::Sender<SubBlock>) -> Result<u64, ConfirmationLayerError> {
        let mut state = self.state.lock().await;

        if self.senders_cl_to_hig.contains_key(&chain_id.0) {
            log("CL", &format!("Chain {} is already registered.", chain_id.0));
            return Err(ConfirmationLayerError::ChainAlreadyRegistered(chain_id));
        }

        self.senders_cl_to_hig.insert(chain_id.0.clone(), sender);
        log("CL", &format!("Channel registered successfully for chain '{}'.", chain_id.0));

        if !state.registered_chains.contains(&chain_id) {
            state.registered_chains.push(chain_id.clone());
            log("CL", &format!("Chain {} added to registered_chains.", chain_id.0));
        }

        Ok(state.current_block_height)
    }

}

#[async_trait::async_trait]
impl ConfirmationLayer for Arc<Mutex<ConfirmationLayerNode>> {
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

    async fn register_chain(&mut self, chain_id: ChainId, sender: mpsc::Sender<SubBlock>) -> Result<u64, ConfirmationLayerError> {
        let mut node = self.lock().await;
        node.register_chain(chain_id, sender).await
    }

}