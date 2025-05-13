mod transaction;
pub mod cat;
mod block;
mod chain;
pub mod communication;

use serde::{Deserialize, Serialize};

/// Unique identifier for a transaction
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionId(pub String);

// Re-export all types
pub use transaction::*;
pub use cat::*;
pub use block::*;
pub use chain::*;
pub use communication::*; 