//! Simple timestamped logging.
//! 
//! Provides a basic logging function that prefixes messages with timestamps.

use std::time::{SystemTime, UNIX_EPOCH};

// ------------------------------------------------------------------------------------------------
// Logging Functions
// ------------------------------------------------------------------------------------------------

/// Logs a message with timestamp and prefix
pub fn log(prefix: &str, message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    println!("[{}] {}: {}", timestamp, prefix, message);
} 