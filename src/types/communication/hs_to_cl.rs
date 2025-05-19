use crate::types::CLTransaction;
use serde::{Deserialize, Serialize};

/// A message from HS to CL requesting a status update transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLTransactionMessage {
    /// The transaction to submit
    pub cl_transaction: CLTransaction,
} 