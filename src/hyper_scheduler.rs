use crate::types::{CAT, CATId, CATStatus, TransactionId, TransactionStatus};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HyperSchedulerError {
    #[error("CAT not found: {0}")]
    CATNotFound(CATId),
    #[error("Transaction not found: {0}")]
    TransactionNotFound(TransactionId),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait HyperScheduler {
    /// Get the current state of a CAT
    async fn get_cat_status(&self, id: CATId) -> Result<CAT, HyperSchedulerError>;
    
    /// Get all pending CATs
    async fn get_pending_cats(&self) -> Result<Vec<CAT>, HyperSchedulerError>;

    /// Submit a transaction status update from the Hyper IG
    async fn submit_transaction_status(&mut self, tx_id: TransactionId, status: TransactionStatus) -> Result<(), HyperSchedulerError>;

    /// Submit a CAT status update to the confirmation layer
    async fn submit_cat_status(&mut self, cat_id: CATId, status: CATStatus) -> Result<(), HyperSchedulerError>;
} 