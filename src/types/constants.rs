use crate::types::ChainId;

pub const CHAIN_1: &str = "chain-1";
pub const CHAIN_2: &str = "chain-2";
pub const CHAIN_3: &str = "chain-3";

/// Chain ID for the first test chain
pub fn chain_1() -> ChainId {
    ChainId(CHAIN_1.to_string())
}

/// Chain ID for the second test chain
pub fn chain_2() -> ChainId {
    ChainId(CHAIN_2.to_string())
}

/// Chain ID for the third test chain
pub fn chain_3() -> ChainId {
    ChainId(CHAIN_3.to_string())
} 