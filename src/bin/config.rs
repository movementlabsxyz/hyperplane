use std::time::Duration;

// Block time configuration
pub const BLOCK_TIME_MILLISECONDS: u64 = 500;
pub const BLOCK_TIME: Duration = Duration::from_millis(BLOCK_TIME_MILLISECONDS);

// CAT (Cross-Chain Atomic Transaction) configuration
pub const CAT_MAX_LIFETIME_BLOCKS: u64 = 10;

// Allow CATs to depend on pending transactions
pub const ALLOW_CAT_PENDING_DEPENDENCIES: bool = false;

// Channel buffer sizes for high-performance communication
pub const CHANNEL_BUFFER_SIZE: usize = 1000;

// Dummy main function to satisfy Rust compiler for bin directory
#[allow(dead_code)]
fn main() {}
 