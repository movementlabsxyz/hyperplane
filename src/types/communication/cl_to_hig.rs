use crate::types::{SubBlock};
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

lazy_static! {
    pub static ref CAT_ID_SUFFIX: &'static str = r"\.CAT_ID:(?P<cat_id>[a-zA-Z0-9_-]+)";

    // Expected formats of the data field of a transaction:
    // REGULAR.credit <receiver> <amount>
    // REGULAR.send <sender> <receiver> <amount>
    // DEPENDENT.credit <receiver> <amount>.CAT_ID:<ID>
    // DEPENDENT.send <sender> <receiver> <amount>.CAT_ID:<ID>
    // CAT.credit <receiver> <amount>
    // CAT.send <sender> <receiver> <amount>
    // STATUS_UPDATE:<StatusLimited>.CAT_ID:<ID>
    pub static ref REGULAR_PATTERN: Regex = Regex::new(r"^REGULAR\.(credit \d+ \d+|send \d+ \d+ \d+)$").unwrap();
    pub static ref DEPENDENT_PATTERN: Regex = Regex::new(&format!(r"^DEPENDENT\.(credit \d+ \d+|send \d+ \d+ \d+){}$", *CAT_ID_SUFFIX)).unwrap();
    pub static ref CAT_PATTERN: Regex = Regex::new(&format!(r"^CAT\.(credit \d+ \d+|send \d+ \d+ \d+)$")).unwrap();
    pub static ref STATUS_UPDATE_PATTERN: Regex = Regex::new(&format!(r"^STATUS_UPDATE:(Success|Failure){}$", *CAT_ID_SUFFIX)).unwrap();
}

