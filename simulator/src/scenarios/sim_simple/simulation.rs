use std::env;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use std::time::{Duration, Instant};
use toml;

// ------------------------------------------------------------------------------------------------
// Configuration Loading
// ------------------------------------------------------------------------------------------------

/// Loads and validates the simple simulation configuration from the TOML file.
/// 
/// This function reads the configuration from config.toml in the sim_simple directory
/// and validates it according to the simple simulation's requirements.
fn load_config() -> Result<crate::config::Config, crate::config::ConfigError> {
    let config_str = fs::read_to_string("simulator/src/scenarios/sim_simple/config.toml")?;
    let config: crate::config::Config = toml::from_str(&config_str)?;
    config.validate()?;
    Ok(config)
}

// ------------------------------------------------------------------------------------------------
// Simulation Entry Point
// ------------------------------------------------------------------------------------------------

/// Runs the simple simulation
pub async fn run_simple_simulation() -> Result<(), crate::config::ConfigError> {
    // Create results directory if it doesn't exist
    fs::create_dir_all("simulator/results/sim_simple").expect("Failed to create results directory");
    fs::create_dir_all("simulator/results/sim_simple/data").expect("Failed to create data directory");
    fs::create_dir_all("simulator/results/sim_simple/figs").expect("Failed to create figures directory");
    
    // Setup logging
    setup_logging();

    // Load configuration
    let config = load_config()?;
    
    // Initialize simulation results from configuration
    let mut results = initialize_simulation_results(&config);

    // Setup test nodes (with zero delays for funding)
    let (_hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = crate::testnodes::setup_test_nodes(
        Duration::from_secs_f64(config.network_config.block_interval),
        &[0, 0], // Zero delays for funding
        config.transaction_config.allow_cat_pending_dependencies,
        config.transaction_config.cat_lifetime_blocks,
    ).await;
    
    // Initialize accounts with initial balance (with zero delays for fast processing)
    crate::network::initialize_accounts(
        &[cl_node.clone()], 
        config.account_config.initial_balance.try_into().unwrap(), 
        config.account_config.num_accounts.try_into().unwrap(),
        Some(&[hig_node_1.clone(), hig_node_2.clone()]),
        config.network_config.block_interval,
    ).await.map_err(|e| crate::config::ConfigError::ValidationError(e.to_string()))?;
    
    // Now set the actual chain delays for the main simulation
    logging::log("SIMULATOR", "Setting actual chain delays for main simulation...");
    let delay_1_time = Duration::from_secs_f64(config.network_config.block_interval * config.network_config.chain_delays[0] as f64);
    let delay_2_time = Duration::from_secs_f64(config.network_config.block_interval * config.network_config.chain_delays[1] as f64);
    hig_node_1.lock().await.set_hs_message_delay(delay_1_time);
    hig_node_2.lock().await.set_hs_message_delay(delay_2_time);
    logging::log("SIMULATOR", &format!("Set chain 1 delay to {} blocks ({:?}) and chain 2 delay to {} blocks ({:?})", 
        config.network_config.chain_delays[0], delay_1_time, config.network_config.chain_delays[1], delay_2_time));

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

/// Runs the simple simulation with automatic plotting
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    use crate::scenarios::utils::run_simulation_with_plotting;
    
    run_simulation_with_plotting(
        || run_simple_simulation(),
        "Simple Simulation",
        "simulator/src/scenarios/sim_simple/plot_results.py"
    ).await
}

// ------------------------------------------------------------------------------------------------
// Logging Setup
// ------------------------------------------------------------------------------------------------

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

// ------------------------------------------------------------------------------------------------
// Results Initialization
// ------------------------------------------------------------------------------------------------

/// Initializes simulation results from configuration
fn initialize_simulation_results(config: &crate::config::Config) -> crate::SimulationResults {
    let mut results = crate::SimulationResults::default();
    results.initial_balance = config.account_config.initial_balance.try_into().unwrap();
    results.num_accounts = config.account_config.num_accounts.try_into().unwrap();
    results.target_tps = config.transaction_config.target_tps as u64;
            results.sim_total_block_number = config.transaction_config.sim_total_block_number.try_into().unwrap();
    results.zipf_parameter = config.transaction_config.zipf_parameter;
    results.ratio_cats = config.transaction_config.ratio_cats;
    results.block_interval = config.network_config.block_interval;
    results.cat_lifetime = config.transaction_config.cat_lifetime_blocks;
    results.initialization_wait_blocks = config.transaction_config.initialization_wait_blocks;
    results.chain_delays = config.network_config.chain_delays.clone();
    results.start_time = Instant::now();

    // Log configuration
    let start_time = Local::now();
    logging::log("SIMULATOR", "=== Simulation Configuration ===");
    logging::log("SIMULATOR", &format!("Start Time: {}", start_time.format("%Y-%m-%d %H:%M:%S")));
    logging::log("SIMULATOR", &format!("Initial Balance: {}", config.account_config.initial_balance));
    logging::log("SIMULATOR", &format!("Number of Accounts: {}", config.account_config.num_accounts));
    logging::log("SIMULATOR", &format!("Target TPS: {}", config.transaction_config.target_tps));
    logging::log("SIMULATOR", &format!("Simulation Total Blocks: {}", config.transaction_config.sim_total_block_number));
    logging::log("SIMULATOR", &format!("Number of Chains: {}", config.network_config.num_chains));
    logging::log("SIMULATOR", &format!("Zipf Parameter: {}", config.transaction_config.zipf_parameter));
    logging::log("SIMULATOR", &format!("Ratio CATs: {}", config.transaction_config.ratio_cats));
    logging::log("SIMULATOR", &format!("CAT Lifetime: {} blocks", results.cat_lifetime));
    logging::log("SIMULATOR", &format!("Initialization Wait Blocks: {}", results.initialization_wait_blocks));
    for (i, delay) in config.network_config.chain_delays.iter().enumerate() {
        logging::log("SIMULATOR", &format!("Chain {} Delay: {} blocks", i + 1, delay));
    }
    logging::log("SIMULATOR", "=============================");

    results
}

// ------------------------------------------------------------------------------------------------
// Simulation Registration
// ------------------------------------------------------------------------------------------------

/// Register this simulation with the simulation registry.
/// 
/// This function provides the configuration needed to register the simple simulation
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::Simple, SimulationConfig {
        name: "Simple Simulation",
        run_fn: Box::new(|| Box::pin(async {
            run_simple_simulation().await
                .map_err(|e| format!("Simple simulation failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_simple/plot_results.py",
    })
} 