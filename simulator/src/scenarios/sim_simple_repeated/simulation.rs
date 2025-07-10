use std::env;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use std::time::{Duration, Instant};
use toml;

// ------------------------------------------------------------------------------------------------
// Configuration Loading
// ------------------------------------------------------------------------------------------------

/// Loads and validates the simple (repeated) simulation configuration from the TOML file.
/// 
/// This function reads the configuration from config.toml in the sim_simple_repeated directory
/// and validates it according to the simple (repeated) simulation's requirements.
fn load_config() -> Result<crate::config::Config, crate::config::ConfigError> {
    let config_str = fs::read_to_string("simulator/src/scenarios/sim_simple_repeated/config.toml")?;
    let config: crate::config::Config = toml::from_str(&config_str)?;
    config.validate()?;
    Ok(config)
}

// ------------------------------------------------------------------------------------------------
// Simulation Entry Point
// ------------------------------------------------------------------------------------------------

/// Runs the simple (repeated) simulation
pub async fn run_simple_and_repeat_simulation() -> Result<(), crate::config::ConfigError> {
    // Create results directory if it doesn't exist
    fs::create_dir_all("simulator/results/sim_simple_repeated").expect("Failed to create results directory");
    fs::create_dir_all("simulator/results/sim_simple_repeated/data").expect("Failed to create data directory");
    fs::create_dir_all("simulator/results/sim_simple_repeated/figs").expect("Failed to create figures directory");
    
    // Setup logging
    setup_logging();

    // Load configuration
    let config = load_config()?;
    
    // Get number of runs from config
    let num_runs = config.repeat_config.num_runs;
    
    // Display simulation name
    println!("Running Simple (repeated) Simulation with {} runs", num_runs);
    
    logging::log("SIMULATOR", &format!("Starting Simple (repeated) Simulation with {} runs", num_runs));
    
    // Run the simulation multiple times
    for run in 1..=num_runs {
        logging::log("SIMULATOR", &format!("=== Starting Run {}/{} ===", run, num_runs));
        
        // Initialize simulation results from configuration
        let mut results = initialize_simulation_results(&config, run);

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
            config.transaction_config.funding_wait_blocks,
        ).await.map_err(|e| crate::config::ConfigError::ValidationError(e.to_string()))?;
        
        // Now set the actual chain delays for the main simulation
        logging::log("SIMULATOR", "Setting actual chain delays for main simulation...");
        let delay_1_time = Duration::from_secs_f64(config.network_config.block_interval * config.network_config.chain_delays[0] as f64);
        let delay_2_time = Duration::from_secs_f64(config.network_config.block_interval * config.network_config.chain_delays[1] as f64);
        hig_node_1.lock().await.set_hs_message_delay(delay_1_time);
        hig_node_2.lock().await.set_hs_message_delay(delay_2_time);
        logging::log("SIMULATOR", &format!("Set chain 1 delay to {} blocks ({:?}) and chain 2 delay to {} blocks ({:?})", 
            config.network_config.chain_delays[0], delay_1_time, config.network_config.chain_delays[1], delay_2_time));

        // Run simulation with run message
        let run_message = format!("Run {}/{}", run, num_runs);
        crate::run_simulation::run_simulation_with_message(
            cl_node,
            vec![hig_node_1, hig_node_2],
            &mut results,
            Some(run_message),
        ).await.map_err(|e| crate::config::ConfigError::ValidationError(e))?;

        // Save this run's results to its own directory
        let run_dir = format!("simulator/results/sim_simple_repeated/data/run_{}", run - 1); // Use 0-based indexing
        results.save_to_directory(&run_dir).await.map_err(|e| crate::config::ConfigError::ValidationError(e))?;
        
        logging::log("SIMULATOR", &format!("=== Completed Run {}/{} ===", run, num_runs));
    }
    
    // Save metadata about the repeated simulation
    save_repeated_simulation_metadata(num_runs as usize, &config).await.map_err(|e| crate::config::ConfigError::ValidationError(e))?;

    // Show final completion progress bar
    use indicatif::{ProgressBar, ProgressStyle};
    let final_progress = ProgressBar::new(num_runs as u64);
    final_progress.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
        .unwrap()
        .progress_chars("+>-"));
    final_progress.set_position(num_runs as u64);
    final_progress.set_message(format!("Run {}/{}", num_runs, num_runs));
    final_progress.finish_with_message(format!("Run {}/{}", num_runs, num_runs));

    // Show completion status
    println!("Simple (repeated) simulation complete");
    logging::log("SIMULATOR", "=== Simple (repeated) Simulation Complete ===");
    logging::log("SIMULATOR", &format!("Total runs completed: {}", num_runs));
    logging::log("SIMULATOR", "Individual run data saved to run_0/, run_1/, etc. directories");
    logging::log("SIMULATOR", "Use Python plotting scripts to analyze and average the results");

    Ok(())
}

// ------------------------------------------------------------------------------------------------
// Metadata Saving
// ------------------------------------------------------------------------------------------------

/// Saves metadata about the repeated simulation
async fn save_repeated_simulation_metadata(num_runs: usize, config: &crate::config::Config) -> Result<(), String> {
    use serde_json;
    
    let metadata = serde_json::json!({
        "simulation_type": "simple_repeated",
        "num_runs": num_runs,
        "parameters": {
            "initial_balance": config.account_config.initial_balance,
            "num_accounts": config.account_config.num_accounts,
            "target_tps": config.transaction_config.target_tps,
            "sim_total_block_number": config.transaction_config.sim_total_block_number,
            "zipf_parameter": config.transaction_config.zipf_parameter,
            "ratio_cats": config.transaction_config.ratio_cats,
            "block_interval": config.network_config.block_interval,
            "chain_delays": config.network_config.chain_delays.clone()
        },
        "note": "Individual run data is stored in run_0/, run_1/, etc. directories. Use Python plotting scripts to analyze and average the results."
    });

    let metadata_file = "simulator/results/sim_simple_repeated/data/metadata.json";
    fs::write(metadata_file, serde_json::to_string_pretty(&metadata).expect("Failed to serialize metadata")).map_err(|e| e.to_string())?;
    logging::log("SIMULATOR", &format!("Saved simulation metadata to {}", metadata_file));

    Ok(())
}

/// Runs the simple (repeated) simulation with automatic plotting
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    use crate::scenarios::utils::run_simulation_with_plotting;
    
    run_simulation_with_plotting(
        || run_simple_and_repeat_simulation(),
        "Simple (repeated) Simulation",
        "simulator/src/scenarios/sim_simple_repeated/plot_results.py"
    ).await
}

// ------------------------------------------------------------------------------------------------
// Logging Setup
// ------------------------------------------------------------------------------------------------

/// Sets up logging if ENABLE_LOGS environment variable is set
fn setup_logging() {
    if env::var("ENABLE_LOGS").is_ok() {
        // Delete existing log file if it exists
        let log_path = "simulator/results/sim_simple_repeated/simulation.log";
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
fn initialize_simulation_results(config: &crate::config::Config, run_number: u32) -> crate::SimulationResults {
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

    // Log configuration for this run
    let start_time = Local::now();
    logging::log("SIMULATOR", &format!("=== Run {} Configuration ===", run_number));
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
/// This function provides the configuration needed to register the simple (repeated) simulation
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SimpleAndRepeat, SimulationConfig {
        name: "Simple (repeated) Simulation",
        run_fn: Box::new(|| Box::pin(async {
            run_simple_and_repeat_simulation().await
                .map_err(|e| format!("Simple (repeated) simulation failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_simple_repeated/plot_results.py",
    })
} 