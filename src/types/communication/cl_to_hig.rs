use crate::types::SubBlock;
use serde::{Deserialize, Serialize};
use regex::Regex;
use lazy_static::lazy_static;

/// A message from CL to HIG containing a new subblock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubBlockMessage {
    /// The subblock to process
    pub subblock: SubBlock,
}

// Define valid data strings for transactions in a subblock
pub enum TransactionData {
    /// A regular non-dependent transaction
    Regular(String),
    /// A regular dependent transaction
    Dependent(String),
    /// A CAT transaction
    CAT(String),
    /// A status update transaction
    StatusUpdate(String),
}

impl TransactionData {
    /// Validates if a transaction data string matches the expected format
    pub fn validate(data: &str) -> Result<TransactionData, String> {
        if CAT_PATTERN.is_match(data) {
            Ok(TransactionData::CAT(data.to_string()))
        } else if STATUS_UPDATE_PATTERN.is_match(data) {
            Ok(TransactionData::StatusUpdate(data.to_string()))
        } else if DEPENDENT_PATTERN.is_match(data) {
            Ok(TransactionData::Dependent(data.to_string()))
        } else if REGULAR_PATTERN.is_match(data) {
            Ok(TransactionData::Regular(data.to_string()))
        } else {
            Err("Invalid transaction data format".to_string())
        }
    }
}


// The following is only a requirement for the simulation.
// Expected formats of the data field of a transaction:
// REGULAR.SIMULATION.<StatusLimited>
// DEPENDENT.SIMULATION.<StatusLimited>
// CAT.SIMULATION.<StatusLimited>.CAT-ID:<ID>
// STATUS_UPDATE.<StatusLimited>.CAT-ID:<ID>
lazy_static! {
    static ref CAT_PATTERN: Regex = Regex::new(r"^CAT\.SIMULATION\.(Success|Failure)\.CAT_ID:[a-zA-Z0-9_-]+$").unwrap();
    static ref STATUS_UPDATE_PATTERN: Regex = Regex::new(r"^STATUS_UPDATE\.(Success|Failure)\.CAT_ID:[a-zA-Z0-9_-]+$").unwrap();
    static ref DEPENDENT_PATTERN: Regex = Regex::new(r"^DEPENDENT\.SIMULATION\.(Success|Failure)$").unwrap();
    static ref REGULAR_PATTERN: Regex = Regex::new(r"^REGULAR\.SIMULATION\.(Success|Failure)$").unwrap();
}