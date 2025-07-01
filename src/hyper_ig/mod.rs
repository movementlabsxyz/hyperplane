use crate::types::{TransactionId, TransactionStatus, Transaction, CATId, CATStatusLimited, SubBlock, ChainId};
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
    #[error("CAT depends on pending transaction: {0}")]
    CATDependsOnPendingTransaction(String),
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

    /// Get the current resolution status of a transaction
    async fn get_resolution_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError>;

    /// Process a subblock of transactions
    async fn process_subblock(&mut self, subblock: SubBlock) -> Result<(), HyperIGError>;

    /// Get the dependencies of a transaction
    async fn get_transaction_dependencies(&self, transaction_id: TransactionId) -> Result<Vec<TransactionId>, HyperIGError>;

    /// Gets the data of a transaction.
    async fn get_transaction_data(&self, tx_id: TransactionId) -> Result<String, anyhow::Error>;

    /// Gets the current state of the chain.
    /// Returns a HashMap containing the current state of all accounts and their balances.
    async fn get_chain_state(&self) -> Result<std::collections::HashMap<String, i64>, anyhow::Error>;

    /// Get the maximum lifetime for a CAT transaction
    async fn get_cat_max_lifetime(&self, cat_id: CATId) -> Result<u64, HyperIGError>;

    /// Get the current block height
    async fn get_current_block_height(&self) -> Result<u64, HyperIGError>;

    /// Get the default CAT lifetime configuration in blocks
    async fn get_cat_lifetime(&self) -> Result<u64, HyperIGError>;

    /// Get the count of transactions with a specific status
    async fn get_transaction_status_count(&self, status: TransactionStatus) -> Result<u64, HyperIGError>;

    /// Get counts of all transaction statuses (Pending, Success, Failure)
    async fn get_transaction_status_counts(&self) -> Result<(u64, u64, u64), HyperIGError>;
}

#[cfg(test)]
mod tests; 