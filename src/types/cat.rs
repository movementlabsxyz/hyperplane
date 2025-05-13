use serde::{Deserialize, Serialize};
use std::fmt;
use super::TransactionId;

/// Unique identifier for a Crosschain Atomic Transaction (CAT)
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CATId(pub String);

/// Status of a Crosschain Atomic Transaction (CAT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CATStatus {
    /// CAT is pending
    Pending,
    /// CAT is successful
    Success,
    /// CAT failed
    Failure,
}

/// The proposed status of a CAT from the Hyper IG to the Hyper Scheduler
/// We use this as we would like to have a reduced set of options (we do not want to have a pending status)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CATStatusLimited {
    /// CAT is successful
    Success,
    /// CAT failed
    Failure,
}

/// A status update for a CAT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CATStatusUpdate {
    /// The ID of the CAT
    pub cat_id: CATId,
    /// The new status
    pub status: CATStatusLimited,
}

/// A Crosschain Atomic Transaction (CAT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CAT {
    /// Unique identifier for this CAT
    pub id: CATId,
    /// The transactions that are part of this CAT
    pub transactions: Vec<TransactionId>,
    /// The status of this CAT
    pub status: CATStatus,
}

impl fmt::Display for CATId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
} 