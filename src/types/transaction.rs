use serde::{Deserialize, Serialize};
use std::fmt;
use super::{ChainId, TransactionId};
use crate::types::communication::cl_to_hig::TransactionData;
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

impl CLTransaction {
    /// Creates a new CLTransaction, ensuring that the `data` string matches expected format
    pub fn new(id: TransactionId, chain_id: ChainId, data: String) -> Result<Self, String> {
        // Use TransactionData's validation logic
        TransactionData::validate(&data)?;
        Ok(CLTransaction { id, chain_id, data })
    }
}

/// A simple transaction type for testing destined to be included in a subblock and the respective chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique identifier for this transaction
    pub id: TransactionId,
    /// The actual transaction data (just a string for now)
    pub data: String,
}

impl Transaction {
    /// Creates a new Transaction, ensuring that the `data` string matches expected format
    pub fn new(id: TransactionId, data: String) -> Result<Self, String> {
        // Use TransactionData's validation logic
        TransactionData::validate(&data)?;
        Ok(Transaction { id, data })
    }
}

/// A status update for a transaction from the Hyper IG to the Hyper Scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusUpdate {
    pub transaction_id: TransactionId,
    pub status: TransactionStatus,
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
} 