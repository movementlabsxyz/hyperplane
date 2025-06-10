use serde::{Deserialize, Serialize};
use std::fmt;
use super::{ChainId, CLTransactionId};
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

/// A simple transaction type for testing destined to be included in a subblock and the respective chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Unique identifier for this transaction
    pub id: TransactionId,
    /// The chain ID of this transaction
    pub chain_id: ChainId,
    /// The chain IDs to which this transaction is destined
    pub constituent_chains: Vec<ChainId>,
    /// The actual transaction data (just a string for now)
    pub data: String,
    /// The ID of the CL transaction this transaction belongs to
    pub cl_id: CLTransactionId,
}

impl Transaction {
    /// Creates a new Transaction, ensuring that the `data` string matches expected format
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for this transaction
    /// * `chain_id` - The chain ID of the target chain
    /// * `constituent_chains` - The chain IDs of the constituent chains
    /// * `data` - The actual transaction data
    /// * `cl_id` - The ID of the CL transaction this transaction belongs to
    pub fn new(id: TransactionId,chain_id: ChainId,constituent_chains: Vec<ChainId>,data: String,cl_id: CLTransactionId) -> Result<Self, String> {
        if constituent_chains.is_empty() {
            return Err("Transaction must have at least one constituent chain".to_string());
        }
        if !constituent_chains.contains(&chain_id) {
            return Err("Target chain must be in constituent chains".to_string());
        }
        // Use TransactionData's validation logic
        TransactionData::validate(&data)?;
        Ok(Self {id,chain_id,constituent_chains,data,cl_id})
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