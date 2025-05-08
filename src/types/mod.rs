use serde::{Deserialize, Serialize};
use std::fmt;
use aptos_types::transaction::SignedTransaction;

/// A unique identifier for a transaction
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionId(pub String);

/// A unique identifier for a chain
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChainId(pub String);

/// A unique identifier for a CAT
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CATId(pub String);

/// A single transaction that can be part of a CAT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: TransactionId,
    pub aptos_tx: SignedTransaction,
    pub destination_chain: ChainId,
}

/// Status of a transaction in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,    /// Transaction is waiting to be processed
    Success,    /// Transaction by the chain itself would be successful
    Failure,    /// Transaction by the chain itself would fail
}

/// Status of a Crosschain Atomic Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CATStatus {
    Pending,    /// CAT is waiting to be processed
    Success,    /// All transactions in the CAT have been shown to be successful
    Failure,    /// One or more transactions in the CAT have failed
}

/// A status update for a transaction from the Hyper IG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusUpdate {
    pub transaction_id: TransactionId,
    pub status: TransactionStatus,
}

/// A status update for a CAT from the Hyper Scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CATStatusUpdate {
    pub cat_id: CATId,
    pub status: CATStatus,
}

/// A Crosschain Atomic Transaction (CAT) that consists of multiple transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CAT {
    pub id: CATId,
    pub transactions: Vec<Transaction>,
    pub status: CATStatus,
    pub conflicts: Vec<CATId>,
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for CATId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for CATStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CATStatus::Pending => write!(f, "Pending"),
            CATStatus::Success => write!(f, "Success"),
            CATStatus::Failure => write!(f, "Failure"),
        }
    }
} 