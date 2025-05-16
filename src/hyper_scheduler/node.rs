use crate::types::{CATId, TransactionId, StatusLimited, CLTransaction, ChainId, CATStatusUpdate};
use super::{HyperScheduler, HyperSchedulerError};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The internal state of the HyperSchedulerNode
pub struct HyperSchedulerState {
    /// Map of CAT IDs to their current status update
    pub cat_statuses: HashMap<CATId, StatusLimited>,
    /// The chain IDs for submitting transactions
    pub chain_ids: HashSet<ChainId>,
}

/// A node that implements the HyperScheduler trait
pub struct HyperSchedulerNode {
    /// The internal state of the node
    pub state: Arc<Mutex<HyperSchedulerState>>,
    /// Receiver for messages from Hyper IG
    pub receiver_from_hig: Option<mpsc::Receiver<CATStatusUpdate>>,
    /// Sender for messages to CL
    pub sender_to_cl: Option<mpsc::Sender<CLTransaction>>,
}

impl Clone for HyperSchedulerNode {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            receiver_from_hig: None, // Can't clone receiver
            sender_to_cl: self.sender_to_cl.clone(),
        }
    }
}

impl HyperSchedulerNode {
    /// Create a new HyperSchedulerNode
    pub fn new(receiver_from_hig: mpsc::Receiver<CATStatusUpdate>, sender_to_cl: mpsc::Sender<CLTransaction>) -> Self {
        Self {
            state: Arc::new(Mutex::new(HyperSchedulerState {
                cat_statuses: HashMap::new(),
                chain_ids: HashSet::new(),
            })),
            receiver_from_hig: Some(receiver_from_hig),
            sender_to_cl: Some(sender_to_cl),
        }
    }

    /// Get a clone of the sender to the confirmation layer
    pub async fn get_sender_to_cl(&self) -> mpsc::Sender<CLTransaction> {
        self.sender_to_cl.as_ref().expect("Sender to CL not set").clone()
    }

    /// Take the receiver and state out of the node for message processing
    pub fn take_receiver_and_state(&mut self) -> (mpsc::Receiver<CATStatusUpdate>, Arc<Mutex<HyperSchedulerState>>) {
        let receiver = self.receiver_from_hig.take().expect("Receiver already taken");
        let state = self.state.clone();
        (receiver, state)
    }

    /// Process messages without holding the node lock
    pub async fn process_messages(hs_node: Arc<Mutex<HyperSchedulerNode>>) {
        // println!("  [HS]   [Message loop task] Attempting to acquire hs_node lock...");
        let mut node = hs_node.lock().await;
        // println!("  [HS]   [Message loop task] Acquired hs_node lock");
        let mut receiver = node.receiver_from_hig.take().expect("Receiver already taken");
        let state = node.state.clone();
        drop(node); // Release the lock before starting the loop
        // println!("  [HS]   [Message loop task] Released hs_node lock");
        
        // Process messages
        while let Some(status_update) = receiver.recv().await {
            println!("  [HS]   [Message loop task] Received status proposal for '{}': {:?}", status_update.cat_id, status_update);
            // TODO need to handle chain id as well 
            // println!("  [HS]   [Message loop task] Attempting to acquire state lock for status update...");
            {
                let mut state = state.lock().await;
                // println!("  [HS]   [Message loop task] Acquired state lock for status update");
                state.cat_statuses.insert(status_update.cat_id.clone(), status_update.status.clone());
                // println!("  [HS]   [Message loop task] Updated state, releasing lock");
            }
            // println!("  [HS]   [Message loop task] Released state lock after status update");
            println!("  [HS]   [Message loop task] Successfully processed status proposal for {}", status_update.cat_id);
            // TODO: we need to send the status update to the CL
            // for now we just send it always (=single chain cats)
            let mut node = hs_node.lock().await;
            if let Err(e) = node.send_cat_status_update(status_update.cat_id.clone(), status_update.status.clone()).await {
                println!("  [HS]   Failed to send status update: {:?}", e);
            }
        }
        println!("  [HS]   [Message loop task] Message processing loop exiting");
    }

    /// Start the message processing loop (deprecated, use process_messages instead)
    pub async fn start(&mut self) {
        let hs_node = Arc::new(Mutex::new(self.clone()));
        Self::process_messages(hs_node).await;
    }

    /// Set the chain ID to use for submitting transactions
    pub async fn set_chain_id(&mut self, chain_id: ChainId) {
        self.state.lock().await.chain_ids.insert(chain_id);
    }

    /// Submit a transaction to the confirmation layer
    pub async fn submit_transaction(&mut self, tx: CLTransaction) -> Result<(), String> {
        println!("  [HS]   submit_transaction called for transaction: id={}, data={}, chain_id={}", tx.id.0, tx.data, tx.chain_id.0);
        if let Some(sender) = &self.sender_to_cl {
            sender.send(tx).await.map_err(|e| e.to_string())
        } else {
            Err("No sender to CL set".to_string())
        }
    }

    /// Get the current block from the confirmation layer
    // TODO: i think we dont need this
    pub async fn get_current_block(&mut self) -> Result<u64, String> {
        if let Some(sender) = &self.sender_to_cl {
            let tx = CLTransaction {
                id: TransactionId("GET_CURRENT_BLOCK".to_string()),
                data: "GET_CURRENT_BLOCK".to_string(),
                chain_id: ChainId("SYSTEM".to_string()),
            };
            sender.send(tx).await.map_err(|e| e.to_string())?;
            Ok(0) // For now, just return 0 since we don't have a response channel
        } else {
            Err("No sender to CL set".to_string())
        }
    }

    /// Get a subblock from the confirmation layer
    // TODO: i think we dont need this
    pub async fn get_subblock(&mut self, chain_id: ChainId, block_num: u64) -> Result<Vec<CLTransaction>, String> {
        if let Some(sender) = &self.sender_to_cl {
            let tx = CLTransaction {
                id: TransactionId(format!("GET_SUBBLOCK_{}", block_num)),
                data: format!("GET_SUBBLOCK_{}", block_num),
                chain_id: chain_id.clone(),
            };
            sender.send(tx).await.map_err(|e| e.to_string())?;
            Ok(vec![]) // For now, just return empty vec since we don't have a response channel
        } else {
            Err("No sender to CL set".to_string())
        }
    }
}

#[async_trait]
impl HyperScheduler for HyperSchedulerNode {
    async fn get_cat_status(&self, id: CATId) -> Result<StatusLimited, HyperSchedulerError> {
        println!("  [HS]   get_cat_status called for {}", id.0);
        // println!("  [HS]   Attempting to acquire state lock for get_cat_status...");
        let result = {
            let state = self.state.lock().await;
            // println!("  [HS]   Acquired state lock for get_cat_status");
            let result = state.cat_statuses.get(&id).cloned();
            // println!("  [HS]   Retrieved status, releasing lock");
            result
        };
        // println!("  [HS]   Released state lock after get_cat_status");
        if let Some(ref status) = result {
            println!("  [HS]   get_cat_status found status for {}: {:?}", id.0, status);
        } else {
            println!("  [HS]   get_cat_status did not find status for '{}'", id.0);
        }
        result.ok_or_else(|| HyperSchedulerError::CATNotFound(id))
    }

    async fn get_pending_cats(&self) -> Result<Vec<CATId>, HyperSchedulerError> {
        Ok(self.state.lock().await.cat_statuses.keys().cloned().collect())
    }

    async fn receive_cat_status_proposal(&mut self, cat_id: CATId, status: StatusLimited) -> Result<(), HyperSchedulerError> {
        println!("  [HS]   receive_cat_status_proposal called for {} with status {:?}", cat_id.0, status);
        let mut state = self.state.lock().await;
        
        // Check if CAT already exists
        if state.cat_statuses.contains_key(&cat_id) {
            println!("  [HS]   CAT {} already exists, rejecting duplicate proposal", cat_id.0);
            return Err(HyperSchedulerError::DuplicateProposal(cat_id));
        }
        
        // Store the status update
        state.cat_statuses.insert(cat_id.clone(), status.clone());
        println!("  [HS]   Status for {} set to {:?}", cat_id.0, status);
        Ok(())
    }

    async fn send_cat_status_update(&mut self, cat_id: CATId, status: StatusLimited) -> Result<(), HyperSchedulerError> {
        println!("  [HS]   send_cat_status_update called for CAT {} with status {:?}", cat_id.0, status);
        // Update the CAT status
        self.state.lock().await.cat_statuses.insert(cat_id.clone(), status.clone());

        // Get chain IDs
        let chain_ids = self.state.lock().await.chain_ids.clone();
        if chain_ids.is_empty() {
            println!("  [HS]   No chain IDs set, cannot send status update");
            return Err(HyperSchedulerError::Internal("No chain IDs set".to_string()));
        }

        let status_str = match status {
            StatusLimited::Success => "STATUS_UPDATE.Success.CAT_ID:".to_string() + &cat_id.0,
            StatusLimited::Failure => "STATUS_UPDATE.Failure.CAT_ID:".to_string() + &cat_id.0,
        };

        // Send the status update to all registered chains
        if let Some(sender) = &self.sender_to_cl {
            for chain_id in chain_ids {
                let tx = CLTransaction {
                    id: TransactionId(cat_id.0.clone()+".UPDATE"),
                    data: status_str.clone(),
                    chain_id: chain_id.clone(),
                };
                println!("  [HS]   Submitting status update transaction to CL: id={}, data={}, chain_id={}", tx.id.0, tx.data, tx.chain_id.0);
                sender.send(tx)
                    .await
                    .map_err(|e| HyperSchedulerError::Internal(e.to_string()))?;
            }
        } else {
            println!("  [HS]   No sender to CL set, cannot send status update");
            return Err(HyperSchedulerError::Internal("No sender to CL set".to_string()));
        }

        Ok(())
    }
} 