use serde::{Deserialize, Serialize};
use std::fmt;
use super::{TransactionId, ChainId};

/// Unique identifier for a Crosschain Atomic Transaction (CAT)
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CATId(pub String);

/// Status of a Crosschain Atomic Transaction (CAT)
/// used in HS to keep track of the status of a CAT
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CATStatus {
    /// CAT is pending
    Pending,
    /// CAT is successful
    Success,
    /// CAT is failed
    Failure,
}

/// The possible final status of a CAT or transaction
/// used for proposals from HIG to HS,  status updates from HS to CL, and to keep track of the status proposals in HS
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CATStatusLimited {
    /// CAT is successful
    Success,
    /// CAT is failed
    Failure,
}

/// A status update for a CAT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CATStatusUpdate {
    /// The ID of the CAT
    pub cat_id: CATId,
    /// The ID of the chain that the status is from
    pub chain_id: ChainId,
    /// The new status
    pub status: CATStatusLimited,
    /// The set of all chains involved in this CAT (including the chain_id that sent this update)
    pub constituent_chains: Vec<ChainId>,
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