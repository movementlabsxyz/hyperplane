use std::env;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use simulator::{
    config::{Config, ConfigError},
    initialize_accounts,
    run_simulation,
    testnodes::setup_test_nodes,
    SimulationResults,
};
use std::time::{Duration, Instant};


// ------------------------------------------------------------------------------------------------
// Main
// ------------------------------------------------------------------------------------------------

/// Main function that orchestrates the simulation setup and execution
#[tokio::main]
async fn main() -> Result<(), ConfigError> {
    // Create results directory if it doesn't exist
    fs::create_dir_all("simulator/results").expect("Failed to create results directory");
    
    // Setup logging
    setup_logging();

    // Load configuration
    let config = Config::load()?;
    
    // Initialize simulation results from configuration
    let mut results = initialize_simulation_results(&config);

    // Setup test nodes
    let (_hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = setup_test_nodes(
        Duration::from_secs_f64(config.block_interval),
        &config.chains.delays,
    ).await;
    // Initialize accounts with initial balance
    initialize_accounts(&[cl_node.clone()], config.initial_balance.try_into().unwrap(), config.num_accounts.try_into().unwrap()).await;

    // Run simulation
    run_simulation(
        cl_node,
        vec![hig_node_1, hig_node_2],
        &mut results,
    ).await.map_err(|e| ConfigError::ValidationError(e))?;

    Ok(())
} 

/// Sets up logging if ENABLE_LOGS environment variable is set
fn setup_logging() {
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
    }
}

/// Initializes simulation results from configuration
fn initialize_simulation_results(config: &Config) -> SimulationResults {
    let mut results = SimulationResults::default();
    results.initial_balance = config.initial_balance.try_into().unwrap();
    results.num_accounts = config.num_accounts.try_into().unwrap();
    results.target_tps = config.target_tps as u64;
    results.duration_seconds = config.duration_seconds.try_into().unwrap();
    results.zipf_parameter = config.zipf_parameter;
    results.ratio_cats = config.ratio_cats;
    results.block_interval = config.block_interval;
    results.cat_lifetime = config.cat_lifetime;
    results.chain_delays = config.chains.delays.clone();
    results.start_time = Instant::now();

    // Log configuration
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
    logging::log("SIMULATOR", &format!("CAT Lifetime: {} blocks", config.cat_lifetime));
    for (i, delay) in config.chains.delays.iter().enumerate() {
        logging::log("SIMULATOR", &format!("Chain {} Delay: {:?}", i + 1, delay));
    }
    logging::log("SIMULATOR", "=============================");

    results
}