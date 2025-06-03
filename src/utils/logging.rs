use std::sync::atomic::{AtomicBool, Ordering};
use std::env;

static ENABLE_LOGGING: AtomicBool = AtomicBool::new(false);

/// Initializes logging based on the HYPERPLANE_LOGGING environment variable.
/// - If HYPERPLANE_LOGGING=true, logging is enabled.
/// - If HYPERPLANE_LOGGING=false or not set, logging is disabled.
/// - To enable logging in tests, run: HYPERPLANE_LOGGING=true cargo test -- --nocapture
pub fn init_logging() {
    match env::var("HYPERPLANE_LOGGING") {
        Ok(value) => {
            match value.as_str() {
                "true" => ENABLE_LOGGING.store(true, Ordering::SeqCst),
                "false" => ENABLE_LOGGING.store(false, Ordering::SeqCst),
                _ => panic!("\nError: HYPERPLANE_LOGGING environment variable must be 'true' or 'false'\n\nTo run the program, use one of:\n  HYPERPLANE_LOGGING=true cargo run\n  HYPERPLANE_LOGGING=false cargo run\n"),
            }
        }
        Err(_) => ENABLE_LOGGING.store(false, Ordering::SeqCst),
    }
}

pub fn log(prefix: &str, message: &str) {
    if ENABLE_LOGGING.load(Ordering::SeqCst) {
        println!("  [{}]   {}", prefix, message);
    }
} 