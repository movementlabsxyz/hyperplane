use std::collections::{HashMap, HashSet};
use async_trait::async_trait;
use crate::types::{TransactionId, TransactionStatus, SubBlockTransaction, CATStatusProposal, TransactionStatusUpdate};
use super::{HyperIG, HyperIGError};

/// A simple node implementation of the HyperIG
pub struct HyperIGNode {
    /// Current status of transactions
    transaction_statuses: HashMap<TransactionId, TransactionStatus>,
    /// Set of pending transaction IDs
    pending_transactions: HashSet<TransactionId>,
    /// Proposed status for CAT transactions
    cat_proposed_statuses: HashMap<TransactionId, CATStatusProposal>,
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
}

#[async_trait]
impl HyperIG for HyperIGNode {
    async fn execute_transaction_wrapper(&mut self, transaction: SubBlockTransaction) -> Result<TransactionStatus, anyhow::Error> {
        match transaction {
            SubBlockTransaction::Regular(tx_wrapper) => {
                // For regular transactions, check if it's a CAT
                if tx_wrapper.is_cat {
                    // CAT transactions always stay pending
                    self.transaction_statuses.insert(tx_wrapper.transaction.id.clone(), TransactionStatus::Pending);
                    self.pending_transactions.insert(tx_wrapper.transaction.id.clone());
                    
                    // Set proposed status based on data
                    let proposed_status = if tx_wrapper.transaction.data == "success" {
                        CATStatusProposal::Success
                    } else {
                        CATStatusProposal::Failure
                    };
                    self.cat_proposed_statuses.insert(tx_wrapper.transaction.id.clone(), proposed_status);
                    
                    Ok(TransactionStatus::Pending)
                } else {
                    // For normal transactions, check if data is dependent
                    // TODO: This is a dummy implementation for testing.
                    if tx_wrapper.transaction.data == "dependent" {
                        self.transaction_statuses.insert(tx_wrapper.transaction.id.clone(), TransactionStatus::Pending);
                        self.pending_transactions.insert(tx_wrapper.transaction.id.clone());
                        Ok(TransactionStatus::Pending)
                    } else {
                        self.transaction_statuses.insert(tx_wrapper.transaction.id.clone(), TransactionStatus::Success);
                        Ok(TransactionStatus::Success)
                    }
                }
            }
            SubBlockTransaction::StatusUpdate(status_update) => {
                // For status updates, update the CAT's status
                let new_status = if status_update.success {
                    TransactionStatus::Success
                } else {
                    TransactionStatus::Failure
                };
                
                // Update the status
                self.transaction_statuses.insert(status_update.cat_id.clone(), new_status.clone());
                
                // Remove from pending if it was there
                self.pending_transactions.remove(&status_update.cat_id);
                
                // Remove from proposed statuses
                self.cat_proposed_statuses.remove(&status_update.cat_id);
                
                Ok(new_status)
            }
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

    async fn submit_cat_status_proposal(&mut self, update: TransactionStatusUpdate) -> Result<(), HyperIGError> {
        // For now, just update the status directly
        self.transaction_statuses.insert(update.transaction_id.clone(), update.status.clone());
        self.pending_transactions.remove(&update.transaction_id);
        self.cat_proposed_statuses.remove(&update.transaction_id);
        
        Ok(())
    }
}

impl HyperIGNode {
    /// Get the proposed status for a CAT transaction
    pub async fn get_proposed_status(&self, transaction_id: TransactionId) -> Result<CATStatusProposal, anyhow::Error> {
        Ok(self.cat_proposed_statuses.get(&transaction_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No proposed status found for transaction"))?)
    }
} 