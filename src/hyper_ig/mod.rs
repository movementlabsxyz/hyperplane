use crate::types::{TransactionId, TransactionStatus, Transaction, CAT, CATId, CATStatusLimited, SubBlock, ChainId};
use async_trait::async_trait;
use thiserror::Error;

pub mod node;
pub use node::HyperIGNode;

#[derive(Debug, Error)]
pub enum HyperIGError {
    #[error("Transaction not found: {0}")]
    TransactionNotFound(TransactionId),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Communication error: {0}")]
    Communication(String),
    #[error("Received subblock with wrong chain ID: expected {expected}, got {received}")]
    WrongChainId { expected: ChainId, received: ChainId },
    #[error("Invalid CAT constituent chains: {0}")]
    InvalidCATConstituentChains(String),
}

/// The Hyper IG is responsible for executing transactions,
/// managing their status, and resolving CAT transactions.
#[async_trait]
pub trait HyperIG: Send + Sync {
    /// Execute a transaction and return its status
    async fn process_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, anyhow::Error>;

    /// Get the current status of a transaction
    async fn get_transaction_status(&self, transaction_id: TransactionId) -> Result<TransactionStatus, anyhow::Error>;

    /// Get all pending transaction IDs
    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, anyhow::Error>;

    /// Submit a CAT status proposal to the Hyper Scheduler
    async fn send_cat_status_proposal(&mut self, cat_id: CATId, status: CATStatusLimited, constituent_chains: Vec<ChainId>) -> Result<(), HyperIGError>;

    /// Resolve the status of a CAT transaction based on hyper_scheduler and sequencer views
    async fn resolve_cat(&mut self, tx: CAT) -> Result<TransactionStatus, HyperIGError>;
    
    /// Get the current resolution status of a transaction
    async fn get_resolution_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError>;

    /// Process a subblock of transactions
    async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError>;

    /// Get the dependencies of a transaction
    async fn get_transaction_dependencies(&self, transaction_id: TransactionId) -> Result<Vec<TransactionId>, HyperIGError>;
}

#[cfg(test)]
mod tests; 