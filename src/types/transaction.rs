use serde::{Deserialize, Serialize};
use std::fmt;
use super::ChainId;
use crate::types::communication::cl_to_hig::TransactionData;

/// Unique identifier for a transaction
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionId(pub String);

/// Status of a transaction
/// used in HIG to keep track of the status of a transaction
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

/// A transaction in the confirmation layer destined to be included in one or more subblocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLTransaction {
    /// Unique identifier for this transaction
    pub id: TransactionId,
    /// The chain IDs to which this transaction is destined
    pub constituent_chains: Vec<ChainId>,
    /// The transactions to be included in the subblocks
    pub transactions: Vec<Transaction>,
}

impl CLTransaction {
    /// Creates a new CLTransaction, ensuring that all transaction data strings match expected format
    pub fn new(id: TransactionId, constituent_chains: Vec<ChainId>, transactions: Vec<Transaction>) -> Result<Self, String> {
        // Validate all transaction data strings
        for tx in &transactions {
            TransactionData::validate(&tx.data)?;
        }
        Ok(CLTransaction { id, constituent_chains, transactions })
    }
}

/// A simple transaction type for testing destined to be included in a subblock and the respective chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Unique identifier for this transaction
    pub id: TransactionId,
    /// The target chain ID of this transaction
    pub target_chain_id: ChainId,
    /// The chain IDs to which this transaction is destined
    pub constituent_chains: Vec<ChainId>,
    /// The actual transaction data (just a string for now)
    pub data: String,
}

impl Transaction {
    /// Creates a new Transaction, ensuring that the `data` string matches expected format
    pub fn new(id: TransactionId, target_chain_id: ChainId, constituent_chains: Vec<ChainId>, data: String) -> Result<Self, String> {
        // Use TransactionData's validation logic
        TransactionData::validate(&data)?;
        Ok(Transaction { id, target_chain_id, constituent_chains, data })
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