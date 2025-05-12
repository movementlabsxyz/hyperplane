use std::collections::{HashMap, HashSet};
use async_trait::async_trait;
use crate::types::{Transaction, TransactionId, TransactionStatus, CATStatusUpdate, CAT, CATId, SubBlock};
use super::{HyperIG, HyperIGError};
use crate::hyper_scheduler::HyperScheduler;

/// A simple node implementation of the HyperIG
pub struct HyperIGNode {
    /// Current status of transactions
    transaction_statuses: HashMap<TransactionId, TransactionStatus>,
    /// Set of pending transaction IDs
    pending_transactions: HashSet<TransactionId>,
    /// Proposed status for CAT transactions
    cat_proposed_statuses: HashMap<TransactionId, CATStatusUpdate>,
    /// The hyper scheduler for submitting CAT status updates
    hyper_scheduler: Option<Box<dyn HyperScheduler>>,
}

impl HyperIGNode {
    /// Create a new HyperIG node
    pub fn new() -> Self {
        Self {
            transaction_statuses: HashMap::new(),
            pending_transactions: HashSet::new(),
            cat_proposed_statuses: HashMap::new(),
            hyper_scheduler: None,
        }
    }

    /// Set the hyper scheduler to use for submitting CAT status updates
    pub fn set_hyper_scheduler(&mut self, hs: Box<dyn HyperScheduler>) {
        self.hyper_scheduler = Some(hs);
    }

    /// Get a reference to the hyper scheduler
    /// TODO: this is a dummy implementation for testing. Later we should have a communication channel between HIG and HS
    pub fn hyper_scheduler(&self) -> Option<&Box<dyn HyperScheduler>> {
        self.hyper_scheduler.as_ref()
    }

    /// Process a subblock
    pub async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError> {
        println!("[HIG] Processing subblock: block_id={}, chain_id={}, tx_count={}", subblock.block_id.0, subblock.chain_id.0, subblock.transactions.len());
        for tx in &subblock.transactions {
            println!("[HIG] Executing transaction: id={}, data={}", tx.id.0, tx.data);
        }
        for tx in subblock.transactions {
            self.execute_transaction(tx).await.map_err(|e| HyperIGError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Handle a CAT transaction
    fn handle_cat_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // CAT transactions always stay pending
        self.transaction_statuses.insert(transaction.id.clone(), TransactionStatus::Pending);
        self.pending_transactions.insert(transaction.id.clone());
        
        // Set proposed status based on data
        // TODO: this is a dummy implementation for testing.
        let proposed_status = if transaction.data == "CAT.SIMULATION.SUCCESS" {
            CATStatusUpdate::Success
        } else {
            CATStatusUpdate::Failure
        };
        self.cat_proposed_statuses.insert(transaction.id.clone(), proposed_status);
        Ok(TransactionStatus::Pending)
    }

    /// Handle a regular transaction
    fn handle_regular_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // check if data is dependent
        // TODO: This is a dummy implementation for testing. for now it stays forever pending until we handle dependency
        if transaction.data == "DEPENDENT" {
            self.transaction_statuses.insert(transaction.id.clone(), TransactionStatus::Pending);
            self.pending_transactions.insert(transaction.id.clone());
            Ok(TransactionStatus::Pending)
        } else {
            self.transaction_statuses.insert(transaction.id.clone(), TransactionStatus::Success);
            Ok(TransactionStatus::Success)
        }
    }

    /// Handle a status update transaction
    fn handle_status_update(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        println!("[HIG] Handling status update: id={}, data={}", transaction.id.0, transaction.data);
        // if data starts with "STATUS_UPDATE.SUCCESS" or "STATUS_UPDATE.FAILURE"
        let new_status = if transaction.data.starts_with("STATUS_UPDATE.SUCCESS") {
            TransactionStatus::Success
        } else if transaction.data.starts_with("STATUS_UPDATE.FAILURE") {
            TransactionStatus::Failure
        } else {
            println!("[HIG] Invalid status update data: {}", transaction.data);
            return Err(anyhow::anyhow!("Invalid status update"));
        };
        
        // Extract the original CAT ID from the data
        let cat_id = if let Some(cat_id) = transaction.data.split("CAT_ID:").nth(1) {
            TransactionId(cat_id.to_string())
        } else {
            println!("[HIG] Could not extract CAT ID from status update data: {}", transaction.data);
            return Err(anyhow::anyhow!("Invalid status update data format"));
        };
        
        // Update the status for the original CAT transaction
        println!("[HIG] Setting transaction {} status to {:?}", cat_id.0, new_status);
        self.transaction_statuses.insert(cat_id.clone(), new_status.clone());
        
        // Remove from pending if it was there
        self.pending_transactions.remove(&cat_id);
        
        // Remove from proposed statuses
        self.cat_proposed_statuses.remove(&cat_id);
        
        // Return the new status
        Ok(new_status)
    }
}

#[async_trait]
impl HyperIG for HyperIGNode {
    async fn execute_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error> {
        // identify whether it is a regular transaction or a status update transaction
        let data = transaction.data.clone();
        let is_status_update = data.starts_with("STATUS_UPDATE");
        let is_cat = data.starts_with("CAT");

        if is_status_update {
            self.handle_status_update(transaction)
        } else if is_cat {  
            // handle CAT transaction
            self.handle_cat_transaction(transaction)            
        } else {
            // handle regular transaction
            self.handle_regular_transaction(transaction)
        }
    }

    async fn get_transaction_status(&self, transaction_id: TransactionId) -> Result<TransactionStatus, anyhow::Error> {
        Ok(self.transaction_statuses.get(&transaction_id)
            .cloned()
            .unwrap_or(TransactionStatus::Pending))
    }

    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error> {
        Ok(self.pending_transactions.iter().cloned().collect())
    }

    async fn send_cat_status_proposal(&mut self, cat_id: CATId, status: CATStatusUpdate) -> Result<(), HyperIGError> {
        let hs = self.hyper_scheduler.as_mut()
            .ok_or_else(|| HyperIGError::Internal("Hyper scheduler not set".to_string()))?;

        // Convert CATId to TransactionId for the proposal
        let tx_id = TransactionId(cat_id.0.clone());
        
        // Store the proposed status locally
        self.cat_proposed_statuses.insert(tx_id.clone(), status.clone());

        // Send the proposal to the hyper scheduler
        hs.receive_cat_status_proposal(tx_id, status)
            .await
            .map_err(|e| HyperIGError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn resolve_transaction(&mut self, tx: CAT) -> Result<TransactionStatus, HyperIGError> {
        // Convert CATId to TransactionId by using the inner String value
        let transaction_id = TransactionId(tx.id.0);
        self.get_resolution_status(transaction_id).await
    }

    async fn get_resolution_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError> {
        Ok(self.transaction_statuses.get(&id)
            .cloned()
            .unwrap_or(TransactionStatus::Pending))
    }
}

impl HyperIGNode {
    /// Get the proposed status for a CAT transaction
    pub async fn get_proposed_status(&self, transaction_id: TransactionId) -> Result<CATStatusUpdate, anyhow::Error> {
        Ok(self.cat_proposed_statuses.get(&transaction_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No proposed status found for transaction"))?)
    }
} 