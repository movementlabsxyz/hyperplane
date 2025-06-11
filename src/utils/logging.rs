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

pub fn log(prefix: &str, message: &str) {
    if !*ENABLE_LOGGING.lock().unwrap() {
        return;
    }

    let log_message = format!("  [{}]   {}\n", prefix, message);
    
    if *LOG_TO_FILE.lock().unwrap() {
        if let Some(file) = &mut *LOG_FILE.lock().unwrap() {
            let _ = file.write_all(log_message.as_bytes());
            let _ = file.flush();
        }
    } else {
        print!("{}", log_message);
    }
} 