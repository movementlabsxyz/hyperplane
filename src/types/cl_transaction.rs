use serde::{Deserialize, Serialize};
use super::{ChainId, Transaction, TransactionId};
use crate::types::communication::cl_to_hig::TransactionData;

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