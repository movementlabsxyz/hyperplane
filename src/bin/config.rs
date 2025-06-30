use std::time::Duration;

// Block time configuration
pub const BLOCK_TIME_MILLISECONDS: u64 = 500;
pub const BLOCK_TIME: Duration = Duration::from_millis(BLOCK_TIME_MILLISECONDS);

// CAT (Cross-Chain Atomic Transaction) configuration
pub const CAT_MAX_LIFETIME_BLOCKS: u64 = 5;

// Dummy main function to satisfy Rust compiler for bin directory
#[allow(dead_code)]
fn main() {}
 