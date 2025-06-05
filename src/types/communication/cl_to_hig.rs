use crate::types::{SubBlock, CATId};
use serde::{Deserialize, Serialize};
use regex::Regex;
use lazy_static::lazy_static;
use anyhow::anyhow;

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

/// Parse a CAT transaction and extract its ID and status
pub fn parse_cat_transaction(data: &str) -> Result<CATId, anyhow::Error> {
    println!("Parsing CAT transaction: {}", data);
    println!("CAT_PATTERN: {}", *CAT_PATTERN);
    println!("CAT_ID_SUFFIX: {}", *CAT_ID_SUFFIX);
    println!("Full pattern being used: {}", format!(r"^CAT\.(credit \d+ \d+|send \d+ \d+ \d+){}$", *CAT_ID_SUFFIX));
    
    // First check if it's a CAT transaction at all
    if !data.starts_with("CAT.") {
        println!("Data does not start with 'CAT.'");
        return Err(anyhow!("Invalid CAT transaction format: {}", data));
    }
    
    // Then try the pattern match
    let is_match = CAT_PATTERN.is_match(data);
    println!("Pattern match result: {}", is_match);
    
    if let Some(captures) = CAT_PATTERN.captures(data) {
        println!("Pattern matched successfully");
        println!("Captures: {:?}", captures);
        
        // Extract the command part from the first capture group
        let command_part = captures.get(1)
            .ok_or_else(|| anyhow!("No command part found in CAT transaction"))?;
        let command = command_part.as_str().split_whitespace().next()
            .ok_or_else(|| anyhow!("No command found in CAT transaction"))?;
        println!("Extracted command: {}", command);
        
        // Extract CAT ID directly from the captures
        let cat_id = captures.name("cat_id")
            .ok_or_else(|| anyhow!("Failed to extract CAT ID"))?;
        let cat_id = CATId(cat_id.as_str().to_string());
        println!("Extracted CAT ID: '{}'", cat_id.0);

        Ok(cat_id)
    } else {
        println!("Failed to match CAT_PATTERN for data: {}", data);
        Err(anyhow!("Invalid CAT transaction format: {}", data))
    }
}

lazy_static! {
    pub static ref CAT_ID_SUFFIX: &'static str = r"\.CAT_ID:(?P<cat_id>[a-zA-Z0-9_-]+)";

    // Expected formats of the data field of a transaction:
    // REGULAR.credit <receiver> <amount>
    // REGULAR.send <sender> <receiver> <amount>
    // DEPENDENT.credit <receiver> <amount>.CAT_ID:<ID>
    // DEPENDENT.send <sender> <receiver> <amount>.CAT_ID:<ID>
    // CAT.credit <receiver> <amount>.CAT_ID:<ID>
    // CAT.send <sender> <receiver> <amount>.CAT_ID:<ID>
    // STATUS_UPDATE:<StatusLimited>.CAT_ID:<ID>
    pub static ref REGULAR_PATTERN: Regex = Regex::new(r"^REGULAR\.(credit \d+ \d+|send \d+ \d+ \d+)$").unwrap();
    pub static ref DEPENDENT_PATTERN: Regex = Regex::new(&format!(r"^DEPENDENT\.(credit \d+ \d+|send \d+ \d+ \d+){}$", *CAT_ID_SUFFIX)).unwrap();
    pub static ref CAT_PATTERN: Regex = Regex::new(&format!(r"^CAT\.(credit \d+ \d+|send \d+ \d+ \d+){}$", *CAT_ID_SUFFIX)).unwrap();
    pub static ref STATUS_UPDATE_PATTERN: Regex = Regex::new(&format!(r"^STATUS_UPDATE:(Success|Failure){}$", *CAT_ID_SUFFIX)).unwrap();
}

