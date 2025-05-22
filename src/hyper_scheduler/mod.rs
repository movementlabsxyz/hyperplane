use crate::types::{CATId, TransactionId, CATStatusLimited, ChainId, CATStatus};
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
    #[error("Duplicate proposal: {0}")]
    DuplicateProposal(CATId),
    #[error("Invalid CAT proposal: {0}")]
    InvalidCATProposal(String),
}

#[async_trait]
pub trait HyperScheduler: Send + Sync {
    /// Get the current status update of a CAT
    async fn get_cat_status(&self, id: CATId) -> Result<CATStatus, HyperSchedulerError>;
    
    /// Get all pending CAT IDs
    async fn get_pending_cats(&self) -> Result<Vec<CATId>, HyperSchedulerError>;

    /// Receive a CAT status proposal from the Hyper IG
    async fn process_cat_status_proposal(&mut self, cat_id: CATId, this_chain_id: ChainId, constituent_chains: Vec<ChainId>, status: CATStatusLimited) -> Result<(), HyperSchedulerError>;

    /// Send a CAT status update to the confirmation layer
    async fn send_cat_status_update(&mut self, cat_id: CATId, constituent_chains: Vec<ChainId>, status: CATStatusLimited) -> Result<(), HyperSchedulerError>;
}

#[cfg(test)]
mod tests; 