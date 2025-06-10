use std::env;
use std::time::Duration;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use simulator::{
    network::{setup_nodes, initialize_accounts},
    simulation::run_simulation,
    account_selector::AccountSelector,
};

// ------------------------------------------------------------------------------------------------
// Constants
// ------------------------------------------------------------------------------------------------

const INITIAL_BALANCE: i64 = 1000;
const NUM_ACCOUNTS: usize = 100;
const TARGET_TPS: f64 = 10.0;
const SIMULATION_DURATION: Duration = Duration::from_secs(6); // 6 seconds

// ------------------------------------------------------------------------------------------------
// Main
// ------------------------------------------------------------------------------------------------

/// Main function that orchestrates the simulation setup and execution
#[tokio::main]
async fn main() {
    // Enable logging if ENABLE_LOGS is set
    if env::var("ENABLE_LOGS").is_ok() {
        // Delete existing log file if it exists
        let log_path = "simulator/results/simulation.log";
        if let Err(e) = fs::remove_file(log_path) {
            // Ignore error if file doesn't exist
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("Error deleting log file: {}", e);
            }
        }

        // Initialize logging with simulation-specific log file
        env::set_var("HYPERPLANE_LOGGING", "true");
        env::set_var("HYPERPLANE_LOG_TO_FILE", "true");
        env::set_var("HYPERPLANE_LOG_FILE", log_path);
        logging::init_logging();

        // Log simulation header with configuration
        let start_time = Local::now();
        logging::log("SIMULATOR", "=== Simulation Configuration ===");
        logging::log("SIMULATOR", &format!("Start Time: {}", start_time.format("%Y-%m-%d %H:%M:%S")));
        logging::log("SIMULATOR", &format!("Initial Balance: {}", INITIAL_BALANCE));
        logging::log("SIMULATOR", &format!("Number of Accounts: {}", NUM_ACCOUNTS));
        logging::log("SIMULATOR", &format!("Target TPS: {}", TARGET_TPS));
        logging::log("SIMULATOR", &format!("Simulation Duration: {} seconds", SIMULATION_DURATION.as_secs()));
        logging::log("SIMULATOR", "=============================");
    }

    // Setup nodes
    let cl_nodes = setup_nodes().await;

    // Initialize accounts
    initialize_accounts(&cl_nodes, INITIAL_BALANCE).await;

    // Create account selector
    let account_selector = AccountSelector::new(NUM_ACCOUNTS);

    // Run simulation
    run_simulation(
        &cl_nodes,
        account_selector,
        TARGET_TPS,
        SIMULATION_DURATION,
    ).await;
} 