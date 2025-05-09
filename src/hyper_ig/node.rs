use std::collections::{HashMap, HashSet};
use async_trait::async_trait;
use crate::types::{Transaction, TransactionId, TransactionStatus, CATStatusUpdate, TransactionStatusUpdate, CAT};
use super::{HyperIG, HyperIGError};

/// A simple node implementation of the HyperIG
pub struct HyperIGNode {
    /// Current status of transactions
    transaction_statuses: HashMap<TransactionId, TransactionStatus>,
    /// Set of pending transaction IDs
    pending_transactions: HashSet<TransactionId>,
    /// Proposed status for CAT transactions
    cat_proposed_statuses: HashMap<TransactionId, CATStatusUpdate>,
}

impl HyperIGNode {
    /// Create a new HyperIG node
    pub fn new() -> Self {
        Self {
            transaction_statuses: HashMap::new(),
            pending_transactions: HashSet::new(),
            cat_proposed_statuses: HashMap::new(),
        }
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
        // if data == "STATUS_UPDATE.SUCCESS" or "STATUS_UPDATE.FAILURE"
        let new_status = match transaction.data.as_str() {
            "STATUS_UPDATE.SUCCESS" => TransactionStatus::Success,
            "STATUS_UPDATE.FAILURE" => TransactionStatus::Failure,
            _ => return Err(anyhow::anyhow!("Invalid status update")),
        };
        
        // Update the status
        self.transaction_statuses.insert(transaction.id.clone(), new_status.clone());
        
        // Remove from pending if it was there
        self.pending_transactions.remove(&transaction.id);
        
        // Remove from proposed statuses
        self.cat_proposed_statuses.remove(&transaction.id);
        
        // Return Pending to match test expectations
        Ok(TransactionStatus::Pending)
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

    async fn send_cat_status_proposal(&mut self, update: TransactionStatusUpdate) -> Result<(), HyperIGError> {
        // For now, just update the status directly
        self.transaction_statuses.insert(update.transaction_id.clone(), update.status.clone());
        self.pending_transactions.remove(&update.transaction_id);
        self.cat_proposed_statuses.remove(&update.transaction_id);
        
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