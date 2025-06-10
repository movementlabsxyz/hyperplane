use std::env;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use simulator::{
    setup_nodes,
    initialize_accounts,
    run_simulation,
    AccountSelector,
    config::{Config, ConfigError},
};

// ------------------------------------------------------------------------------------------------
// Main
// ------------------------------------------------------------------------------------------------

/// Main function that orchestrates the simulation setup and execution
#[tokio::main]
async fn main() -> Result<(), ConfigError> {
    // Load configuration
    let config = Config::load()?;
    
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
        logging::log("SIMULATOR", &format!("Initial Balance: {}", config.simulation.initial_balance));
        logging::log("SIMULATOR", &format!("Number of Accounts: {}", config.simulation.num_accounts));
        logging::log("SIMULATOR", &format!("Target TPS: {}", config.simulation.target_tps));
        logging::log("SIMULATOR", &format!("Simulation Duration: {} seconds", config.simulation.duration_seconds));
        logging::log("SIMULATOR", &format!("Block Interval: {} seconds", config.simulation.block_interval_seconds));
        logging::log("SIMULATOR", &format!("Number of Chains: {}", config.network.num_chains));
        logging::log("SIMULATOR", &format!("Zipf Parameter: {}", config.simulation.zipf_parameter));
        logging::log("SIMULATOR", "=============================");
    }
    
    // Setup nodes
    let cl_nodes = setup_nodes().await;
    
    // Initialize accounts
    initialize_accounts(&cl_nodes, config.simulation.initial_balance).await;
    
    // Create account selector
    let account_selector = AccountSelector::new(config.simulation.num_accounts, config.simulation.zipf_parameter);
    
    // Run simulation
    run_simulation(
        &cl_nodes,
        account_selector,
        config.simulation.target_tps,
        config.get_duration(),
    ).await;

    Ok(())
} 