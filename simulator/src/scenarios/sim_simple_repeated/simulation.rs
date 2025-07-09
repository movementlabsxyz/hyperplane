use std::env;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use std::time::{Duration, Instant};
use toml;
use std::collections::HashMap;

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
    let num_runs = config.repeat_config.as_ref()
        .map(|rc| rc.num_runs)
        .unwrap_or(1);
    
    // Display simulation name and create progress bar
    println!("Running Simple (repeated) Simulation with {} runs", num_runs);
    use indicatif::{ProgressBar, ProgressStyle};
    let progress_bar = ProgressBar::new(num_runs as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
        .unwrap()
        .progress_chars("+>-"));
    
    logging::log("SIMULATOR", &format!("Starting Simple (repeated) Simulation with {} runs", num_runs));
    
    // Store results from all runs
    let mut all_run_results: Vec<crate::SimulationResults> = Vec::new();
    
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

        // Store results from this run
        all_run_results.push(results);
        
        // Update progress bar
        progress_bar.inc(1);
        progress_bar.set_message(format!("Run {}/{}", run, num_runs));
        
        logging::log("SIMULATOR", &format!("=== Completed Run {}/{} ===", run, num_runs));
    }
    
    // Average the results across all runs
    let averaged_results = average_results(all_run_results);
    
    // Save averaged results
    averaged_results.save_to_directory("simulator/results/sim_simple_repeated").await.map_err(|e| crate::config::ConfigError::ValidationError(e))?;

    // Finish progress bar
    progress_bar.finish_with_message("Simple (repeated) Simulation Complete");

    // Show completion status
    println!("Simple (repeated) simulation complete");
    logging::log("SIMULATOR", "=== Simple (repeated) Simulation Complete ===");
    logging::log("SIMULATOR", &format!("Total runs completed: {}", num_runs));
    logging::log("SIMULATOR", &format!("Average total transactions sent: {}", averaged_results.transactions_sent));
    logging::log("SIMULATOR", &format!("Average CAT transactions: {}", averaged_results.cat_transactions));
    logging::log("SIMULATOR", &format!("Average regular transactions: {}", averaged_results.regular_transactions));

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
// Results Averaging
// ------------------------------------------------------------------------------------------------

/// Averages results across multiple simulation runs
fn average_results(all_results: Vec<crate::SimulationResults>) -> crate::SimulationResults {
    if all_results.is_empty() {
        return crate::SimulationResults::default();
    }
    
    let num_runs = all_results.len();
    let mut averaged = all_results[0].clone();
    
    // Average the time series data
    averaged.chain_1_pending = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_pending).collect()
    );
    averaged.chain_2_pending = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_pending).collect()
    );
    averaged.chain_1_success = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_success).collect()
    );
    averaged.chain_2_success = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_success).collect()
    );
    averaged.chain_1_failure = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_failure).collect()
    );
    averaged.chain_2_failure = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_failure).collect()
    );
    
    // Average CAT-specific data
    averaged.chain_1_cat_pending = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_cat_pending).collect()
    );
    averaged.chain_2_cat_pending = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_cat_pending).collect()
    );
    averaged.chain_1_cat_success = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_cat_success).collect()
    );
    averaged.chain_2_cat_success = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_cat_success).collect()
    );
    averaged.chain_1_cat_failure = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_cat_failure).collect()
    );
    averaged.chain_2_cat_failure = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_cat_failure).collect()
    );
    
    // Average regular transaction data
    averaged.chain_1_regular_pending = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_regular_pending).collect()
    );
    averaged.chain_2_regular_pending = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_regular_pending).collect()
    );
    averaged.chain_1_regular_success = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_regular_success).collect()
    );
    averaged.chain_2_regular_success = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_regular_success).collect()
    );
    averaged.chain_1_regular_failure = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_regular_failure).collect()
    );
    averaged.chain_2_regular_failure = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_regular_failure).collect()
    );
    
    // Average locked keys data
    averaged.chain_1_locked_keys = average_time_series_data(
        all_results.iter().map(|r| &r.chain_1_locked_keys).collect()
    );
    averaged.chain_2_locked_keys = average_time_series_data(
        all_results.iter().map(|r| &r.chain_2_locked_keys).collect()
    );
    
    // Average scalar values
    averaged.transactions_sent = all_results.iter().map(|r| r.transactions_sent).sum::<u64>() / num_runs as u64;
    averaged.cat_transactions = all_results.iter().map(|r| r.cat_transactions).sum::<u64>() / num_runs as u64;
    averaged.regular_transactions = all_results.iter().map(|r| r.regular_transactions).sum::<u64>() / num_runs as u64;
    
    // Average execution time
    let total_duration: Duration = all_results.iter().map(|r| r.start_time.elapsed()).sum();
    averaged.start_time = Instant::now() - (total_duration / num_runs as u32);
    
    logging::log("SIMULATOR", &format!("Averaged results across {} runs", num_runs));
    
    averaged
}

/// Averages time series data across multiple runs
fn average_time_series_data(all_data: Vec<&Vec<(u64, u64)>>) -> Vec<(u64, u64)> {
    if all_data.is_empty() {
        return Vec::new();
    }
    
    // Create a map to store sums and counts for each block height
    let mut height_sums: HashMap<u64, u64> = HashMap::new();
    let mut height_counts: HashMap<u64, u64> = HashMap::new();
    
    // Sum up all values for each block height
    for data in all_data {
        for &(height, count) in data {
            *height_sums.entry(height).or_insert(0) += count;
            *height_counts.entry(height).or_insert(0) += 1;
        }
    }
    
    // Calculate averages and sort by block height
    let mut averaged_data: Vec<(u64, u64)> = height_sums
        .into_iter()
        .map(|(height, sum)| {
            let count = height_counts[&height];
            (height, sum / count)
        })
        .collect();
    
    averaged_data.sort_by_key(|&(height, _)| height);
    averaged_data
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