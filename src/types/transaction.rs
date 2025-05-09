use serde::{Deserialize, Serialize};
use std::fmt;
use super::{ChainId, TransactionId, CATStatusUpdate};

/// Status of a transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    /// Transaction is pending
    Pending,
    /// Transaction is successful and accepted
    Success,
    /// Transaction failed
    /// NOTE: we distinguish not between failed due to execution or due to dependency
    Failure,
}

/// A transaction in the confirmation layer destined to be included in a subblock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLTransaction {
    /// Unique identifier for this transaction
    pub id: TransactionId,
    /// The chain ID
    pub chain_id: ChainId,
    /// The transaction data
    pub data: String,
}

/// A simple transaction type for testing destined to be included in a subblock and the respective chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique identifier for this transaction
    pub id: TransactionId,
    /// The actual transaction data (just a string for now)
    pub data: String,
}

/// A status update for a transaction from the Hyper IG to the Hyper Scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusUpdate {
    pub transaction_id: TransactionId,
    pub status: TransactionStatus,
}

/// A transaction that updates the status of a CAT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdateTransaction {
    /// The ID of the CAT being updated
    pub cat_id: TransactionId,
    /// The new status of the CAT
    pub status: CATStatusUpdate,
    /// The chain this status update is for
    pub chain_id: ChainId,
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
} 