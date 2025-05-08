use crate::types::{CAT, CATId, CATStatus, TransactionId, CATStatusProposal};
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

    /// Receive a CAT status proposal from the Hyper IG
    async fn receive_cat_status_proposal(&mut self, tx_id: TransactionId, status: CATStatusProposal) -> Result<(), HyperSchedulerError>;

    /// Submit a CAT status update to the confirmation layer
    async fn submit_cat_status(&mut self, cat_id: CATId, status: CATStatus) -> Result<(), HyperSchedulerError>;
} 