use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::env;

static LOG_FILE: Lazy<Mutex<Option<std::fs::File>>> = Lazy::new(|| Mutex::new(None));
static ENABLE_LOGGING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));
static LOG_TO_FILE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));

/// Initializes logging by opening the log file.
pub fn init_logging() {
    // Check if we're running tests
    let is_test = env::var("CARGO_PKG_NAME").is_err() || env::var("TEST").is_ok();
    
    // For tests, default to terminal logging unless explicitly set to file
    // For normal runs, default to file logging unless explicitly set to terminal
    let default_to_file = !is_test;
    
    // Check if logging is enabled
    let enable_logging = env::var("HYPERPLANE_LOGGING")
        .map(|v| v == "true")
        .unwrap_or(true);  // Default to enabled
    
    // Check if we should log to file
    let log_to_file = env::var("HYPERPLANE_LOG_TO_FILE")
        .map(|v| v == "true")
        .unwrap_or(default_to_file);
    
    *ENABLE_LOGGING.lock().unwrap() = enable_logging;
    *LOG_TO_FILE.lock().unwrap() = log_to_file;

    if enable_logging && log_to_file {
        let log_file = env::var("HYPERPLANE_LOG_FILE").unwrap_or_else(|_| "hyperplane.log".to_string());
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Failed to open log file");
        *LOG_FILE.lock().unwrap() = Some(file);
    }
}

/// Initializes logging with configuration from config file.
/// 
/// # Arguments
/// * `enabled` - Whether logging is enabled
/// * `log_to_file` - Whether to log to file
/// * `log_file_path` - Optional log file path (if None, uses default)
pub fn init_logging_with_config(enabled: bool, log_to_file: bool, log_file_path: Option<String>) {
    *ENABLE_LOGGING.lock().unwrap() = enabled;
    *LOG_TO_FILE.lock().unwrap() = log_to_file;

    if enabled && log_to_file {
        let log_file = log_file_path.unwrap_or_else(|| "hyperplane.log".to_string());
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Failed to open log file");
        *LOG_FILE.lock().unwrap() = Some(file);
    }
}

/// Resets the logging state by closing the current log file and clearing static state.
/// This prevents state persistence between simulation runs.
pub fn reset_logging() {
    // Close the current log file if it exists
    if let Some(_) = &mut *LOG_FILE.lock().unwrap() {
        *LOG_FILE.lock().unwrap() = None;
    }
    
    // Reset the logging flags to defaults
    *ENABLE_LOGGING.lock().unwrap() = true;
    *LOG_TO_FILE.lock().unwrap() = true;
}

pub fn log(prefix: &str, message: &str) {
    // Check if logging is enabled first - avoid unnecessary work
    let enabled = *ENABLE_LOGGING.lock().unwrap();
    if !enabled {
        return;
    }

    // Only do string formatting and further operations if logging is enabled
    let log_message = format!("  [{}]   {}\n", prefix, message);
    
    let log_to_file = *LOG_TO_FILE.lock().unwrap();
    if log_to_file {
        if let Some(file) = &mut *LOG_FILE.lock().unwrap() {
            let _ = file.write_all(log_message.as_bytes());
            let _ = file.flush();
        }
    } else {
        print!("{}", log_message);
    }
} 