use crate::types::{Transaction, TransactionId, TransactionStatus, TransactionStatusUpdate};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HyperIGError {
    #[error("Transaction not found: {0}")]
    TransactionNotFound(TransactionId),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait HyperIG {
    /// Execute a transaction and determine its status
    async fn execute_transaction(&mut self, transaction: Transaction) -> Result<TransactionStatus, HyperIGError>;
    
    /// Get the current status of a transaction
    async fn get_transaction_status(&self, id: TransactionId) -> Result<TransactionStatus, HyperIGError>;
    
    /// Get all pending transactions
    async fn get_pending_transactions(&self) -> Result<Vec<TransactionId>, HyperIGError>;

    /// Submit a transaction status update to the Hyper Scheduler
    async fn submit_transaction_status(&mut self, update: TransactionStatusUpdate) -> Result<(), HyperIGError>;
} 