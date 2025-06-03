use std::sync::atomic::{AtomicBool, Ordering};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static ENABLE_LOGGING: AtomicBool = AtomicBool::new(false);
static LOG_TO_FILE: AtomicBool = AtomicBool::new(false);
static LOG_FILE: Lazy<Mutex<Option<std::fs::File>>> = Lazy::new(|| Mutex::new(None));

/// Initializes logging based on environment variables:
/// - HYPERPLANE_LOGGING: enables/disables logging (true/false)
/// - HYPERPLANE_LOG_TO_FILE: controls whether logs go to file or stdout (true/false)
pub fn init_logging() {
    match env::var("HYPERPLANE_LOGGING") {
        Ok(value) => {
            match value.as_str() {
                "true" => {
                    ENABLE_LOGGING.store(true, Ordering::SeqCst);
                    // Check if we should log to file
                    if env::var("HYPERPLANE_LOG_TO_FILE").unwrap_or_else(|_| "true".to_string()) == "true" {
                        LOG_TO_FILE.store(true, Ordering::SeqCst);
                        // Open log file
                        let file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("hyperplane.log")
                            .expect("Failed to open log file");
                        *LOG_FILE.lock().unwrap() = Some(file);
                    }
                },
                "false" => ENABLE_LOGGING.store(false, Ordering::SeqCst),
                _ => panic!("\nError: HYPERPLANE_LOGGING environment variable must be 'true' or 'false'\n\nTo run the program, use one of:\n  HYPERPLANE_LOGGING=true cargo run\n  HYPERPLANE_LOGGING=false cargo run\n"),
            }
        }
        Err(_) => ENABLE_LOGGING.store(false, Ordering::SeqCst),
    }
}

pub fn log(prefix: &str, message: &str) {
    if ENABLE_LOGGING.load(Ordering::SeqCst) {
        let log_message = format!("  [{}]   {}\n", prefix, message);
        
        if LOG_TO_FILE.load(Ordering::SeqCst) {
            // Write to file only
            if let Some(file) = &mut *LOG_FILE.lock().unwrap() {
                if let Err(e) = file.write_all(log_message.as_bytes()) {
                    eprintln!("Failed to write to log file: {}", e);
                }
                if let Err(e) = file.flush() {
                    eprintln!("Failed to flush log file: {}", e);
                }
            }
        } else {
            // Write to stdout only
            print!("{}", log_message);
        }
    }
} 