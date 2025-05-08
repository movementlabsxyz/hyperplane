use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::{TransactionId, TransactionStatus, TransactionStatusUpdate, CATStatusProposal, TransactionWrapper};
use super::{HyperIG, HyperIGError};

pub struct HyperIGNode {
    /// Map of transaction IDs to their status
    transaction_statuses: Arc<RwLock<HashMap<TransactionId, TransactionStatus>>>,
    /// Map of CAT transaction IDs to their proposed status
    cat_proposed_statuses: Arc<RwLock<HashMap<TransactionId, CATStatusProposal>>>,
}

impl HyperIGNode {
    pub fn new() -> Self {
        Self {
            transaction_statuses: Arc::new(RwLock::new(HashMap::new())),
            cat_proposed_statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the proposed status for a CAT transaction
    pub async fn get_proposed_status(&self, id: TransactionId) -> Option<CATStatusProposal> {
        self.cat_proposed_statuses.read().await.get(&id).cloned()
    }
}

#[async_trait::async_trait]
impl HyperIG for HyperIGNode {
    async fn execute_transaction_wrapper(&mut self, transaction_wrapper: TransactionWrapper) -> Result<TransactionStatus, HyperIGError> {
        if transaction_wrapper.is_cat {
            // For CAT transactions, always keep status as pending
            let status = TransactionStatus::Pending;
            self.transaction_statuses.write().await.insert(transaction_wrapper.transaction.id.clone(), status.clone());
            
            // Set proposed_status based on data 
            // TODO: this is a dummy implementation for testing.
            let proposed_status = if transaction_wrapper.transaction.data == "success" {
                CATStatusProposal::Success
            } else {
                CATStatusProposal::Failure
            };
            self.cat_proposed_statuses.write().await.insert(transaction_wrapper.transaction.id.clone(), proposed_status);
            
            Ok(status)
        } else {
            // For normal transactions, check the data
            // TODO: this is a dummy implementation for testing. We assume if it is not dependent it cannot fail.
            let status = if transaction_wrapper.transaction.data == "dependent" {
                TransactionStatus::Pending
            } else {
                TransactionStatus::Success
            };
            self.transaction_statuses.write().await.insert(transaction_wrapper.transaction.id.clone(), status.clone());
            Ok(status)
        }
    }

    async fn get_transaction_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError> {
        self.transaction_statuses.read().await
            .get(&id)
            .cloned()
            .ok_or_else(|| HyperIGError::TransactionNotFound(id))
    }

    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, HyperIGError> {
        Ok(self.transaction_statuses.read().await
            .iter()
            .filter(|(_, status)| matches!(status, TransactionStatus::Pending))
            .map(|(id, _)| id.clone())
            .collect())
    }

    async fn submit_cat_status_proposal(&mut self, update: TransactionStatusUpdate) -> Result<(), HyperIGError> {
        self.transaction_statuses.write().await
            .insert(update.transaction_id.clone(), update.status);
        
        // If this was a CAT transaction, remove it from proposed statuses
        self.cat_proposed_statuses.write().await.remove(&update.transaction_id);
        
        Ok(())
    }
} 