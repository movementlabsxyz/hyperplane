use crate::types::{CATId, TransactionId, CATStatusUpdate};
use async_trait::async_trait;
use thiserror::Error;

pub mod node;
pub use node::HyperSchedulerNode;

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
pub trait HyperScheduler: Send + Sync {
    /// Get the current status update of a CAT
    async fn get_cat_status(&self, id: CATId) -> Result<CATStatusUpdate, HyperSchedulerError>;
    
    /// Get all pending CAT IDs
    async fn get_pending_cats(&self) -> Result<Vec<CATId>, HyperSchedulerError>;

    /// Receive a CAT status proposal from the Hyper IG
    async fn receive_cat_status_proposal(&mut self, tx_id: TransactionId, status: CATStatusUpdate) -> Result<(), HyperSchedulerError>;

    /// Send a CAT status update to the confirmation layer
    async fn send_cat_status_update(&mut self, cat_id: CATId, status: CATStatusUpdate) -> Result<(), HyperSchedulerError>;
} 