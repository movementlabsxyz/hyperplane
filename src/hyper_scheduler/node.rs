use crate::types::{CATId, TransactionId, CATStatusLimited, CLTransaction, ChainId, CATStatusUpdate};
use crate::confirmation_layer::ConfirmationLayer;
use super::{HyperScheduler, HyperSchedulerError};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The internal state of the HyperSchedulerNode
struct HyperSchedulerState {
    /// Map of CAT IDs to their current status update
    cat_statuses: HashMap<CATId, CATStatusLimited>,
    /// The confirmation layer for submitting transactions
    confirmation_layer: Option<Box<dyn ConfirmationLayer>>,
    /// The chain IDs for submitting transactions
    chain_ids: HashSet<ChainId>,
    /// Sender for messages to CL
    sender_to_cl: Option<mpsc::Sender<CLTransaction>>,
}

/// A node that implements the HyperScheduler trait
pub struct HyperSchedulerNode {
    /// The internal state of the node
    state: Arc<Mutex<HyperSchedulerState>>,
    /// Receiver for messages from Hyper IG
    receiver_from_hig: Option<mpsc::Receiver<CATStatusUpdate>>,
}

impl HyperSchedulerNode {
    /// Create a new HyperSchedulerNode
    pub fn new(receiver_from_hig: mpsc::Receiver<CATStatusUpdate>, sender_to_cl: mpsc::Sender<CLTransaction>) -> Self {
        Self {
            state: Arc::new(Mutex::new(HyperSchedulerState {
                cat_statuses: HashMap::new(),
                confirmation_layer: None,
                chain_ids: HashSet::new(),
                sender_to_cl: Some(sender_to_cl),
            })),
            receiver_from_hig: Some(receiver_from_hig),
        }
    }

    /// Get a clone of the sender to the confirmation layer
    pub async fn get_sender_to_cl(&self) -> mpsc::Sender<CLTransaction> {
        self.state.lock().await.sender_to_cl.as_ref().expect("Sender to CL not set").clone()
    }

    /// Start the message processing loop
    pub async fn start(&mut self) {
        println!("[HS] Message processing loop started");
        let mut receiver = self.receiver_from_hig.take().expect("Receiver already taken");
        let state = self.state.clone();
        
        while let Some(status_update) = receiver.recv().await {
            println!("[HS] Received status proposal for {}: {:?}", status_update.cat_id, status_update);
            let mut state = state.lock().await;
            state.cat_statuses.insert(status_update.cat_id.clone(), status_update.status.clone());
            println!("[HS] Successfully processed status proposal for {}", status_update.cat_id);
        }
        println!("[HS] Message processing loop exiting");
    }

    /// Set the confirmation layer to use for submitting transactions
    pub async fn set_confirmation_layer(&mut self, cl: Box<dyn ConfirmationLayer>) {
        self.state.lock().await.confirmation_layer = Some(cl);
    }

    /// Set the chain ID to use for submitting transactions
    pub async fn set_chain_id(&mut self, chain_id: ChainId) {
        self.state.lock().await.chain_ids.insert(chain_id);
    }

    /// Submit a transaction to the confirmation layer
    pub async fn submit_transaction(&mut self, tx: CLTransaction) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if let Some(cl) = &mut state.confirmation_layer {
            cl.submit_transaction(tx).await.map_err(|e| e.to_string())
        } else {
            Err("No confirmation layer set".to_string())
        }
    }

    /// Get the current block from the confirmation layer
    pub async fn get_current_block(&mut self) -> Result<u64, String> {
        let mut state = self.state.lock().await;
        if let Some(cl) = &mut state.confirmation_layer {
            cl.get_current_block().await.map_err(|e| e.to_string())
        } else {
            Err("No confirmation layer set".to_string())
        }
    }

    /// Get a subblock from the confirmation layer
    pub async fn get_subblock(&mut self, chain_id: ChainId, block_num: u64) -> Result<Vec<CLTransaction>, String> {
        let mut state = self.state.lock().await;
        if let Some(cl) = &mut state.confirmation_layer {
            cl.get_subblock(chain_id.clone(), block_num).await
                .map(|subblock| {
                    subblock.transactions.into_iter()
                        .map(|tx| CLTransaction {
                            id: tx.id,
                            data: tx.data,
                            chain_id: chain_id.clone(),
                        })
                        .collect()
                })
                .map_err(|e| e.to_string())
        } else {
            Err("No confirmation layer set".to_string())
        }
    }
}

#[async_trait]
impl HyperScheduler for HyperSchedulerNode {
    async fn get_cat_status(&self, id: CATId) -> Result<CATStatusLimited, HyperSchedulerError> {
        println!("[HS] get_cat_status called for {}", id.0);
        let result = self.state.lock().await.cat_statuses.get(&id)
            .cloned();
        if let Some(ref status) = result {
            println!("[HS] get_cat_status found status for {}: {:?}", id.0, status);
        } else {
            println!("[HS] get_cat_status did not find status for {}", id.0);
        }
        result.ok_or_else(|| HyperSchedulerError::CATNotFound(id))
    }

    async fn get_pending_cats(&self) -> Result<Vec<CATId>, HyperSchedulerError> {
        Ok(self.state.lock().await.cat_statuses.keys().cloned().collect())
    }

    async fn receive_cat_status_proposal(&mut self, cat_id: CATId, status: CATStatusLimited) -> Result<(), HyperSchedulerError> {
        println!("[HS] receive_cat_status_proposal called for {} with status {:?}", cat_id.0, status);
        // Store the status update
        self.state.lock().await.cat_statuses.insert(cat_id.clone(), status.clone());
        println!("[HS] Status for {} set to {:?}", cat_id.0, status);
        Ok(())
    }

    async fn send_cat_status_update(&mut self, cat_id: CATId, status: CATStatusLimited) -> Result<(), HyperSchedulerError> {
        println!("[HS] send_cat_status_update called for CAT {} with status {:?}", cat_id.0, status);
        // Update the CAT status
        self.state.lock().await.cat_statuses.insert(cat_id.clone(), status.clone());

        // Submit a CLtransaction to the confirmation layer if available
        if let Some(cl) = &mut self.state.lock().await.confirmation_layer {
            if self.state.lock().await.chain_ids.is_empty() {
                println!("[HS] No chain IDs set, cannot send status update");
                return Err(HyperSchedulerError::Internal("No chain IDs set".to_string()));
            }

            let status_str = match status {
                CATStatusLimited::Success => "STATUS_UPDATE.SUCCESS.CAT_ID:".to_string() + &cat_id.0,
                CATStatusLimited::Failure => "STATUS_UPDATE.FAILURE.CAT_ID:".to_string() + &cat_id.0,
            };

            // Send the status update to all registered chains
            for chain_id in &self.state.lock().await.chain_ids {
                let tx = CLTransaction {
                    id: TransactionId(cat_id.0.clone()+".UPDATE"),
                    data: status_str.clone(),
                    chain_id: chain_id.clone(),
                };
                println!("[HS] Submitting status update transaction to CL: id={}, data={}, chain_id={}", tx.id.0, tx.data, tx.chain_id.0);
                cl.submit_transaction(tx)
                    .await
                    .map_err(|e| HyperSchedulerError::Internal(e.to_string()))?;
            }
        } else {
            println!("[HS] No confirmation layer set, cannot send status update");
        }

        Ok(())
    }
} 