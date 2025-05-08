use crate::types::{TransactionId, TransactionStatus, TransactionStatusUpdate, SubBlockTransaction};
use async_trait::async_trait;
use thiserror::Error;

mod node;
pub use node::HyperIGNode;

#[derive(Debug, Error)]
pub enum HyperIGError {
    #[error("Transaction not found: {0}")]
    TransactionNotFound(TransactionId),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// The Hyper IG is responsible for executing transactions
/// and managing their status.
#[async_trait]
pub trait HyperIG: Send + Sync {
    /// Execute a transaction and return its status
    async fn execute_transaction_wrapper(&mut self, transaction: SubBlockTransaction) -> Result<TransactionStatus, anyhow::Error>;

    /// Get the current status of a transaction
    async fn get_transaction_status(&self, transaction_id: TransactionId) -> Result<TransactionStatus, anyhow::Error>;

    /// Get all pending transaction IDs
    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error>;

    /// Submit a cat status proposal to the Hyper Scheduler
    async fn submit_cat_status_proposal(&mut self, update: TransactionStatusUpdate) -> Result<(), HyperIGError>;
} 