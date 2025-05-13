use serde::{Deserialize, Serialize};
use std::fmt;
use super::BlockId;

/// Unique identifier for a chain
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChainId(pub String);

/// Registration information for a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRegistration {
    /// The ID of the chain
    pub chain_id: ChainId,
    /// The name of the chain
    pub name: String,
    /// The URL of the chain's RPC endpoint
    pub rpc_url: String,
    /// The block at which this chain registered
    pub registration_block: BlockId,
    /// Whether this chain is currently active
    pub active: bool,
}

impl Default for ChainRegistration {
    fn default() -> Self {
        Self {
            chain_id: ChainId("".to_string()),
            name: String::new(),
            rpc_url: String::new(),
            registration_block: BlockId("0".to_string()),
            active: false,
        }
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
} 