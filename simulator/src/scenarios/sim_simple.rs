use std::env;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use std::time::{Duration, Instant};

/// Runs the simple simulation
pub async fn run_simple_simulation() -> Result<(), crate::config::ConfigError> {
    // Create results directory if it doesn't exist
    fs::create_dir_all("simulator/results/sim_simple").expect("Failed to create results directory");
    fs::create_dir_all("simulator/results/sim_simple/data").expect("Failed to create data directory");
    fs::create_dir_all("simulator/results/sim_simple/figs").expect("Failed to create figures directory");
    
    // Setup logging
    setup_logging();

    // Load configuration
    let config = crate::config::Config::load()?;
    
    // Initialize simulation results from configuration
    let mut results = initialize_simulation_results(&config);

    // Setup test nodes
    let (_hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = crate::testnodes::setup_test_nodes(
        Duration::from_secs_f64(config.network.block_interval),
        &config.network.chain_delays,
    ).await;
    
    // Initialize accounts with initial balance
    crate::network::initialize_accounts(&[cl_node.clone()], config.num_accounts.initial_balance.try_into().unwrap(), config.num_accounts.num_accounts.try_into().unwrap()).await;

    // Run simulation
    crate::run_simulation::run_simulation(
        cl_node,
        vec![hig_node_1, hig_node_2],
        &mut results,
    ).await.map_err(|e| crate::config::ConfigError::ValidationError(e))?;

    // Save results
    results.save().await.map_err(|e| crate::config::ConfigError::ValidationError(e))?;

    Ok(())
}

/// Sets up logging if ENABLE_LOGS environment variable is set
fn setup_logging() {
    if env::var("ENABLE_LOGS").is_ok() {
        // Delete existing log file if it exists
        let log_path = "simulator/results/sim_simple/simulation.log";
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
fn initialize_simulation_results(config: &crate::config::Config) -> crate::SimulationResults {
    let mut results = crate::SimulationResults::default();
    results.initial_balance = config.num_accounts.initial_balance.try_into().unwrap();
    results.num_accounts = config.num_accounts.num_accounts.try_into().unwrap();
    results.target_tps = config.transactions.target_tps as u64;
    results.duration_seconds = config.transactions.duration_seconds.try_into().unwrap();
    results.zipf_parameter = config.transactions.zipf_parameter;
    results.ratio_cats = config.transactions.ratio_cats;
    results.block_interval = config.network.block_interval;
    results.cat_lifetime = config.transactions.cat_lifetime_blocks;
    results.initialization_wait_blocks = config.transactions.initialization_wait_blocks;
    results.chain_delays = config.network.chain_delays.clone();
    results.start_time = Instant::now();

    // Log configuration
    let start_time = Local::now();
    logging::log("SIMULATOR", "=== Simulation Configuration ===");
    logging::log("SIMULATOR", &format!("Start Time: {}", start_time.format("%Y-%m-%d %H:%M:%S")));
    logging::log("SIMULATOR", &format!("Initial Balance: {}", config.num_accounts.initial_balance));
    logging::log("SIMULATOR", &format!("Number of Accounts: {}", config.num_accounts.num_accounts));
    logging::log("SIMULATOR", &format!("Target TPS: {}", config.transactions.target_tps));
    logging::log("SIMULATOR", &format!("Simulation Duration: {} seconds", config.transactions.duration_seconds));
    logging::log("SIMULATOR", &format!("Number of Chains: {}", config.network.num_chains));
    logging::log("SIMULATOR", &format!("Zipf Parameter: {}", config.transactions.zipf_parameter));
    logging::log("SIMULATOR", &format!("Ratio CATs: {}", config.transactions.ratio_cats));
    logging::log("SIMULATOR", &format!("CAT Lifetime: {} blocks", results.cat_lifetime));
    logging::log("SIMULATOR", &format!("Initialization Wait Blocks: {}", results.initialization_wait_blocks));
    for (i, delay) in config.network.chain_delays.iter().enumerate() {
        logging::log("SIMULATOR", &format!("Chain {} Delay: {:?}", i + 1, delay));
    }
    logging::log("SIMULATOR", "=============================");

    results
} 