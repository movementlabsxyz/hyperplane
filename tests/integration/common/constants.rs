use hyperplane::types::ChainId;

pub const CHAIN_1: &str = "chain-1";
pub const CHAIN_2: &str = "chain-2";

pub fn chain_1() -> ChainId {
    ChainId(CHAIN_1.to_string())
}

pub fn chain_2() -> ChainId {
    ChainId(CHAIN_2.to_string())
} 