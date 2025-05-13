use serde::{Deserialize, Serialize};
use std::fmt;
use super::{ChainId, Transaction};

/// Unique identifier for a block
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockId(pub String);

/// A sub-block that can be included in a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubBlock {
    /// The block number this subBlock belongs to
    pub block_id: u64,
    /// The chain this subBlock is for
    pub chain_id: ChainId,
    /// The transactions in this sub-block
    pub transactions: Vec<Transaction>,
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
} 