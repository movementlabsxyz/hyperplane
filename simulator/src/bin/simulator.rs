use std::env;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use simulator::{
    setup_nodes,
    initialize_accounts,
    run_simulation,
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
        logging::log("SIMULATOR", &format!("Initial Balance: {}", config.initial_balance));
        logging::log("SIMULATOR", &format!("Number of Accounts: {}", config.num_accounts));
        logging::log("SIMULATOR", &format!("Target TPS: {}", config.target_tps));
        logging::log("SIMULATOR", &format!("Simulation Duration: {} seconds", config.duration_seconds));
        logging::log("SIMULATOR", &format!("Number of Chains: {}", config.chains.num_chains));
        logging::log("SIMULATOR", &format!("Zipf Parameter: {}", config.zipf_parameter));
        logging::log("SIMULATOR", &format!("Ratio CATs: {}", config.ratio_cats));
        for (i, delay) in config.chains.delays.iter().enumerate() {
            logging::log("SIMULATOR", &format!("Chain {} Delay: {} seconds", i + 1, delay));
        }
        logging::log("SIMULATOR", "=============================");
    }
    
    // Setup nodes with chain-specific delays
    let chain_delays: Vec<f64> = (0..config.chains.num_chains)
        .map(|i| config.chains.get_chain_delay(i).as_secs_f64())
        .collect();
    let cl_nodes = setup_nodes(&config.chains.get_chain_ids(), &chain_delays, config.block_interval).await;
    
    // Initialize accounts with initial balance
    initialize_accounts(&cl_nodes, config.initial_balance.try_into().unwrap(), config.num_accounts.try_into().unwrap()).await;
    
    // Run simulation
    run_simulation(
        cl_nodes,
        config.duration_seconds.try_into().unwrap(),
        config.initial_balance.try_into().unwrap(),
        config.num_accounts.try_into().unwrap(),
        config.target_tps as u64,
        config.zipf_parameter,
        chain_delays,
        config.block_interval,
        config.ratio_cats,
    ).await.map_err(|e| ConfigError::ValidationError(e))?;

    Ok(())
} 