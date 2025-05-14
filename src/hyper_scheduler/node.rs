use crate::types::{CATId, TransactionId, CATStatusLimited, CLTransaction, ChainId, CATStatusUpdate};
use crate::confirmation_layer::ConfirmationLayer;
use super::{HyperScheduler, HyperSchedulerError};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

/// A node that implements the HyperScheduler trait
pub struct HyperSchedulerNode {
    /// Map of CAT IDs to their current status update
    cat_statuses: HashMap<CATId, CATStatusLimited>,
    /// The confirmation layer for submitting transactions
    confirmation_layer: Option<Box<dyn ConfirmationLayer>>,
    /// The chain IDs for submitting transactions
    chain_ids: HashSet<ChainId>,
    /// Receiver for messages from Hyper IG
    receiver_from_hig: Option<mpsc::Receiver<CATStatusUpdate>>,
    /// Sender for messages to CL
    sender_to_cl: Option<mpsc::Sender<CLTransaction>>,
}

impl HyperSchedulerNode {
    /// Create a new HyperSchedulerNode
    pub fn new(receiver_from_hig: mpsc::Receiver<CATStatusUpdate>, sender_to_cl: mpsc::Sender<CLTransaction>) -> Self {
        Self {
            cat_statuses: HashMap::new(),
            confirmation_layer: None,
            chain_ids: HashSet::new(),
            receiver_from_hig: Some(receiver_from_hig),
            sender_to_cl: Some(sender_to_cl),
        }
    }

    /// Get a clone of the sender to the confirmation layer
    pub fn get_sender_to_cl(&self) -> mpsc::Sender<CLTransaction> {
        self.sender_to_cl.as_ref().expect("Sender to CL not set").clone()
    }

    /// Start the message processing loop
    pub async fn start(&mut self) {
        let mut receiver = self.receiver_from_hig.take().expect("Receiver already taken");
        while let Some(status_update) = receiver.recv().await {
            println!("[HS] Received status proposal for {}: {:?}", status_update.cat_id, status_update);
            if let Err(e) = self.receive_cat_status_proposal(status_update.cat_id.clone(), status_update.status.clone()).await {
                println!("[HS] Failed to process status proposal for {}: {}", status_update.cat_id, e);
            }
        }
    }

    /// Set the confirmation layer to use for submitting transactions
    pub fn set_confirmation_layer(&mut self, cl: Box<dyn ConfirmationLayer>) {
        self.confirmation_layer = Some(cl);
    }

    /// Set the chain ID to use for submitting transactions
    pub fn set_chain_id(&mut self, chain_id: ChainId) {
        self.chain_ids.insert(chain_id);
    }

    /// Get a reference to the confirmation layer
    pub fn confirmation_layer(&self) -> Option<&Box<dyn ConfirmationLayer>> {
        self.confirmation_layer.as_ref()
    }

    /// Get a mutable reference to the confirmation layer
    pub fn confirmation_layer_mut(&mut self) -> Option<&mut Box<dyn ConfirmationLayer>> {
        self.confirmation_layer.as_mut()
    }
}

#[async_trait::async_trait]
impl HyperScheduler for HyperSchedulerNode {
    async fn get_cat_status(&self, id: CATId) -> Result<CATStatusLimited, HyperSchedulerError> {
        self.cat_statuses.get(&id)
            .cloned()
            .ok_or_else(|| HyperSchedulerError::CATNotFound(id))
    }

    async fn get_pending_cats(&self) -> Result<Vec<CATId>, HyperSchedulerError> {
        Ok(self.cat_statuses.keys().cloned().collect())
    }

    async fn receive_cat_status_proposal(&mut self, cat_id: CATId, status: CATStatusLimited) -> Result<(), HyperSchedulerError> {
        // Store the status update
        self.cat_statuses.insert(cat_id, status);
        Ok(())
    }

    async fn send_cat_status_update(&mut self, cat_id: CATId, status: CATStatusLimited) -> Result<(), HyperSchedulerError> {
        println!("[HS] send_cat_status_update called for CAT {} with status {:?}", cat_id.0, status);
        // Update the CAT status
        self.cat_statuses.insert(cat_id.clone(), status.clone());

        // Submit a CLtransaction to the confirmation layer if available
        if let Some(cl) = &mut self.confirmation_layer {
            if self.chain_ids.is_empty() {
                println!("[HS] No chain IDs set, cannot send status update");
                return Err(HyperSchedulerError::Internal("No chain IDs set".to_string()));
            }

            let status_str = match status {
                CATStatusLimited::Success => "STATUS_UPDATE.SUCCESS.CAT_ID:".to_string() + &cat_id.0,
                CATStatusLimited::Failure => "STATUS_UPDATE.FAILURE.CAT_ID:".to_string() + &cat_id.0,
            };

            // Send the status update to all registered chains
            for chain_id in &self.chain_ids {
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