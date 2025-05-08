use crate::types::{CAT, TransactionId, TransactionStatus};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Transaction not found: {0}")]
    TransactionNotFound(TransactionId),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait Resolver {
    /// Resolve the status of a transaction based on hyper_scheduler and sequencer views
    async fn resolve_transaction(&mut self, tx: CAT) -> Result<TransactionStatus, ResolverError>;
    
    /// Get the current resolution status of a transaction
    async fn get_resolution_status(&self, id: TransactionId) -> Result<TransactionStatus, ResolverError>;
} 