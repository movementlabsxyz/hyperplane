//! Account selection statistics tracking.
//! 
//! Records and reports which accounts were selected as senders and receivers during simulations.

use std::collections::HashMap;
use serde_json;

// ------------------------------------------------------------------------------------------------
// Data Structures
// ------------------------------------------------------------------------------------------------

/// Tracks account selection statistics for senders and receivers
#[derive(Clone, Debug)]
pub struct AccountSelectionStats {
    sender_counts: HashMap<u64, u64>,
    receiver_counts: HashMap<u64, u64>,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl AccountSelectionStats {
    /// Creates a new AccountSelectionStats instance
    pub fn new() -> Self {
        Self {
            sender_counts: HashMap::new(),
            receiver_counts: HashMap::new(),
        }
    }
    
    /// Records a transaction between sender and receiver accounts
    pub fn record_transaction(&mut self, sender: u64, receiver: u64) {
        *self.sender_counts.entry(sender).or_insert(0) += 1;
        *self.receiver_counts.entry(receiver).or_insert(0) += 1;
    }
    
    /// Returns sorted counts for senders and receivers
    pub fn get_sorted_counts(&self) -> (Vec<(u64, u64)>, Vec<(u64, u64)>) {
        let mut sender_counts: Vec<_> = self.sender_counts.clone().into_iter().collect();
        let mut receiver_counts: Vec<_> = self.receiver_counts.clone().into_iter().collect();
        
        sender_counts.sort_by(|a, b| a.0.cmp(&b.0));
        receiver_counts.sort_by(|a, b| a.0.cmp(&b.0));
        
        (sender_counts, receiver_counts)
    }
    
    /// Converts statistics to JSON format for serialization
    pub fn to_json(&self) -> (serde_json::Value, serde_json::Value) {
        let (sender_counts, receiver_counts) = self.get_sorted_counts();
        
        let sender_json = serde_json::json!({
            "sender_selection": sender_counts.iter().map(|(account, transactions)| {
                serde_json::json!({
                    "account": account,
                    "transactions": transactions
                })
            }).collect::<Vec<_>>()
        });

        let receiver_json = serde_json::json!({
            "receiver_selection": receiver_counts.iter().map(|(account, transactions)| {
                serde_json::json!({
                    "account": account,
                    "transactions": transactions
                })
            }).collect::<Vec<_>>()
        });

        (sender_json, receiver_json)
    }
} 