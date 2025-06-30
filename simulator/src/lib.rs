pub mod account_selection;
pub mod zipf_account_selection;
pub mod run_simulation;
pub mod simulation_results;
pub mod network;
pub mod config;
pub mod logging;
pub mod testnodes;
pub mod interface;

pub use account_selection::AccountSelectionStats;
pub use zipf_account_selection::AccountSelector;
pub use run_simulation::run_simulation;
pub use simulation_results::SimulationResults;
pub use network::initialize_accounts;
pub use testnodes::*;
pub use interface::{SimulatorInterface, SimulationType};

// Re-export the run_simple_simulation function
pub async fn run_simple_simulation() -> Result<(), crate::config::ConfigError> {
    use std::env;
    use std::fs;
    use chrono::Local;
    use hyperplane::utils::logging;
    use std::time::{Duration, Instant};
    
    // Create results directory if it doesn't exist
    fs::create_dir_all("simulator/results").expect("Failed to create results directory");
    
    // Setup logging
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

    // Load configuration
    let config = crate::config::Config::load()?;
    
    // Initialize simulation results from configuration
    let mut results = crate::SimulationResults::default();
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

    // Setup test nodes
    let (_hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = crate::testnodes::setup_test_nodes(
        Duration::from_secs_f64(config.block_interval),
        &config.chains.delays,
    ).await;
    
    // Initialize accounts with initial balance
    crate::network::initialize_accounts(&[cl_node.clone()], config.initial_balance.try_into().unwrap(), config.num_accounts.try_into().unwrap()).await;

    // Run simulation
    crate::run_simulation::run_simulation(
        cl_node,
        vec![hig_node_1, hig_node_2],
        &mut results,
    ).await.map_err(|e| crate::config::ConfigError::ValidationError(e))?;

    Ok(())
} 