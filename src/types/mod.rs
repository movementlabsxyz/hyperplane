use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// A unique identifier for a transaction
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionId(pub String);

/// A simple transaction type for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique identifier for this transaction
    pub id: TransactionId,
    /// The chain this transaction is for
    pub chain_id: ChainId,
    /// The actual transaction data (just a string for now)
    pub data: String,
    /// When this transaction was created
    pub timestamp: Duration,
}

/// A unique identifier for a chain
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChainId(pub String);

/// A unique identifier for a Crosschain Atomic Transaction (CAT)
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CATId(pub String);

/// A unique identifier for a block
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockId(pub u64);

/// A subBlock containing transactions for a specific chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubBlock {
    /// The block ID this subBlock belongs to
    pub block_id: BlockId,
    /// The chain this subBlock is for
    pub chain_id: ChainId,
    /// The transactions in this subBlock
    pub transactions: Vec<Transaction>,
}

/// Registration information for a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRegistration {
    /// The chain ID
    pub chain_id: ChainId,
    /// The block at which this chain registered
    pub registration_block: BlockId,
    /// Whether this chain is currently active
    pub active: bool,
}

/// Status of a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending execution
    Pending,
    /// Transaction has been executed successfully
    Executed,
    /// Transaction failed to execute
    Failed(String),
    /// Transaction was cancelled
    Cancelled,
}

/// Status of a Crosschain Atomic Transaction (CAT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CATStatus {
    /// CAT is pending execution
    Pending,
    /// CAT has been executed successfully
    Executed,
    /// CAT failed to execute
    Failed(String),
    /// CAT was cancelled
    Cancelled,
}

/// A status update for a transaction from the Hyper IG to the Hyper Scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusUpdate {
    pub transaction_id: TransactionId,
    pub status: TransactionStatus,
}

/// A status update for a CAT from the Hyper Scheduler to the confirmation layer
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
            CATStatus::Executed => write!(f, "Executed"),
            CATStatus::Failed(reason) => write!(f, "Failed: {}", reason),
            CATStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
} 