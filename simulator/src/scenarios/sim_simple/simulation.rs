use std::fs;

use chrono::Local;
use hyperplane::utils::logging;
use hyperplane::hyper_ig::HyperIG;
use std::time::{Duration, Instant};
use toml;
use serde_json;

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
    
    // Load configuration
    let config = load_config()?;
    
    // Setup logging with configuration
    setup_logging(&config);
    
    // Get number of runs from config
    let num_runs = config.simulation_config.num_runs;
    
    // Write metadata.json for Python averaging script
    let metadata = serde_json::json!({
        "num_runs": num_runs,
        "num_simulations": 1,
        "parameters": {
            "initial_balance": config.account_config.initial_balance,
            "num_accounts": config.account_config.num_accounts,
            "target_tpb": config.transaction_config.target_tpb,
            "sim_total_block_number": config.simulation_config.sim_total_block_number,
            "zipf_parameter": config.transaction_config.zipf_parameter,
            "ratio_cats": config.transaction_config.ratio_cats,
            "block_interval": config.network_config.block_interval,
            "cat_lifetime_blocks": config.transaction_config.cat_lifetime_blocks,
            "chain_delays": config.network_config.chain_delays,
        }
    });
    std::fs::write("simulator/results/sim_simple/data/metadata.json", 
                   serde_json::to_string_pretty(&metadata).unwrap())
        .expect("Failed to write metadata.json");
    
    // Copy config.toml to data directory for reference
    std::fs::copy("simulator/src/scenarios/sim_simple/config.toml", 
                  "simulator/results/sim_simple/data/config.toml")
        .expect("Failed to copy config.toml");
    
    // Display simulation name and create progress bar
    println!("Running Simple Simulation");
    use indicatif::{ProgressBar, ProgressStyle};
    let progress_bar = ProgressBar::new(1); // 1 simulation, not num_runs
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
        .unwrap()
        .progress_chars("+>-"));
    
    // Store results for all runs
    let mut all_results = Vec::new();

    // Run the simulation multiple times
    for run in 1..=num_runs {
        logging::log("SIMULATOR", &format!("=== Starting Run {}/{} ===", run, num_runs));
        
        // Initialize simulation results from configuration
        let mut results = initialize_simulation_results(&config);

        logging::log("SIMULATOR", "Setting up test nodes with preloaded accounts...");
        // Setup test nodes with preloaded accounts from config
        let (hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = crate::testnodes::setup_test_nodes(
            Duration::from_secs_f64(config.network_config.block_interval),
            &[0.0, 0.0], // Zero delays for funding
            config.transaction_config.allow_cat_pending_dependencies,
            config.transaction_config.cat_lifetime_blocks,
            config.account_config.num_accounts.try_into().unwrap(), // Preload accounts from config
            config.account_config.initial_balance.try_into().unwrap(), // Preload value from config
            config.network_config.channel_buffer_size, // Channel buffer size from config
        ).await;
        
        logging::log("SIMULATOR", &format!("Test nodes setup complete with {} accounts preloaded with {} tokens each", 
            config.account_config.num_accounts, config.account_config.initial_balance));
        
        // Query and log account balances to verify preloading
        logging::log("SIMULATOR", "=== Verifying Preloaded Account Balances ===");
        
        // Check chain-1 account balances
        let chain_1_state = hig_node_1.lock().await.get_chain_state().await.unwrap();
        logging::log("SIMULATOR", &format!("Chain-1 state: {} accounts with balances", chain_1_state.len()));
        
        // Log first few account balances as examples
        let mut sorted_accounts: Vec<_> = chain_1_state.iter().collect();
        sorted_accounts.sort_by_key(|(account_id, _)| account_id.parse::<u32>().unwrap_or(0));
        
        for (account_id, balance) in sorted_accounts.iter().take(10) {
            logging::log("SIMULATOR", &format!("Chain-1 Account {}: {} tokens", account_id, balance));
        }
        if sorted_accounts.len() > 10 {
            logging::log("SIMULATOR", &format!("... and {} more accounts", sorted_accounts.len() - 10));
        }
        
        // Check chain-2 account balances
        let chain_2_state = hig_node_2.lock().await.get_chain_state().await.unwrap();
        logging::log("SIMULATOR", &format!("Chain-2 state: {} accounts with balances", chain_2_state.len()));
        
        // Log first few account balances as examples
        let mut sorted_accounts: Vec<_> = chain_2_state.iter().collect();
        sorted_accounts.sort_by_key(|(account_id, _)| account_id.parse::<u32>().unwrap_or(0));
        
        for (account_id, balance) in sorted_accounts.iter().take(10) {
            logging::log("SIMULATOR", &format!("Chain-2 Account {}: {} tokens", account_id, balance));
        }
        if sorted_accounts.len() > 10 {
            logging::log("SIMULATOR", &format!("... and {} more accounts", sorted_accounts.len() - 10));
        }
        
        logging::log("SIMULATOR", "=== Account Balance Verification Complete ===");
        
        // Now set the actual chain delays for the main simulation
        logging::log("SIMULATOR", "Setting actual chain delays for main simulation...");
        let delay_1_time = Duration::from_secs_f64(config.network_config.block_interval * config.network_config.chain_delays[0] as f64);
        let delay_2_time = Duration::from_secs_f64(config.network_config.block_interval * config.network_config.chain_delays[1] as f64);
        hig_node_1.lock().await.set_hs_message_delay(delay_1_time);
        hig_node_2.lock().await.set_hs_message_delay(delay_2_time);
        logging::log("SIMULATOR", &format!("Set chain 1 delay to {} blocks ({:?}) and chain 2 delay to {} blocks ({:?})", 
            config.network_config.chain_delays[0], delay_1_time, config.network_config.chain_delays[1], delay_2_time));

        // Run simulation
        let run_message = format!("Run {}/{}", run, num_runs);
        let simulation_result = crate::run_simulation::run_simulation_with_message_and_retries(
            cl_node.clone(),
            vec![hig_node_1.clone(), hig_node_2.clone()],
            &mut results,
            Some(run_message),
            None, // No retry count needed
        ).await;

        // Check if simulation failed
        if let Err(e) = simulation_result {
            let error_context = format!(
                "Simple simulation failed during run {}/{}: {}",
                run, num_runs, e
            );
            return Err(crate::config::ConfigError::ValidationError(error_context));
        }

        // Shutdown nodes between runs to prevent memory leak
        if run < num_runs {
            logging::log("SIMULATOR", "Shutting down nodes between runs to clear state...");
            
            // Shutdown HIG nodes
            hyperplane::hyper_ig::node::HyperIGNode::shutdown(hig_node_1.clone()).await;
            hyperplane::hyper_ig::node::HyperIGNode::shutdown(hig_node_2.clone()).await;
            
            // Shutdown CL node
            hyperplane::confirmation_layer::node::ConfirmationLayerNode::shutdown(cl_node.clone()).await;
            
            // Shutdown HS node
            hyperplane::hyper_scheduler::node::HyperSchedulerNode::shutdown(hs_node.clone()).await;
            
            logging::log("SIMULATOR", "Node shutdown complete");
        }

        // Save this run's results to its own directory
        let run_dir = format!("simulator/results/sim_simple/data/sim_0/run_{}", run - 1);
        let save_result = results.save_to_directory(&run_dir).await;
        
        if let Err(e) = save_result {
            let error_context = format!(
                "Simple simulation failed to save results for run {}/{}: {}",
                run, num_runs, e
            );
            return Err(crate::config::ConfigError::ValidationError(error_context));
        }

        // Success!
        all_results.push(results);
        logging::log("SIMULATOR", &format!("=== Completed Run {}/{} ===", run, num_runs));
    }

    // Complete progress bar (increment once per simulation, not per run)
    progress_bar.inc(1);
    progress_bar.set_message("Simple Simulation Complete");
    progress_bar.finish_with_message("Simple Simulation Complete");

    // Show completion status
    println!("Simple simulation complete");
    logging::log("SIMULATOR", "=== Simple Simulation Complete ===");
    logging::log("SIMULATOR", &format!("Total runs completed: {}", all_results.len()));
    if let Some(last_result) = all_results.last() {
        logging::log("SIMULATOR", &format!("Total transactions sent: {}", last_result.transactions_sent));
        logging::log("SIMULATOR", &format!("CAT transactions: {}", last_result.cat_transactions));
        logging::log("SIMULATOR", &format!("Regular transactions: {}", last_result.regular_transactions));
    }

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

/// Sets up logging with configuration
fn setup_logging(config: &crate::config::Config) {
    // Delete existing log file if it exists and logging is enabled
    if config.logging_config.log_to_file {
        let log_path = "simulator/results/sim_simple/simulation.log";
        if let Err(e) = fs::remove_file(log_path) {
            // Ignore error if file doesn't exist
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("Error deleting log file: {}", e);
            }
        }

        // Initialize logging with configuration
        logging::init_logging_with_config(
            true, // enabled
            true, // log_to_file
            Some(log_path.to_string())
        );
    } else {
        // Initialize logging with configuration (no file logging)
        logging::init_logging_with_config(
            false, // enabled
            false, // log_to_file
            None
        );
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
            results.target_tpb = config.transaction_config.target_tpb as u64;
    results.sim_total_block_number = config.simulation_config.sim_total_block_number.try_into().unwrap();
    results.zipf_parameter = config.transaction_config.zipf_parameter;
    results.ratio_cats = config.transaction_config.ratio_cats;
    results.block_interval = config.network_config.block_interval;
    results.cat_lifetime = config.transaction_config.cat_lifetime_blocks;
    results.initialization_wait_blocks = config.simulation_config.initialization_wait_blocks;
    results.chain_delays = config.network_config.chain_delays.clone();
    results.start_time = Instant::now();

    // Log configuration
    let start_time = Local::now();
    logging::log("SIMULATOR", "=== Simulation Configuration ===");
    logging::log("SIMULATOR", &format!("Start Time: {}", start_time.format("%Y-%m-%d %H:%M:%S")));
    logging::log("SIMULATOR", &format!("Initial Balance: {}", config.account_config.initial_balance));
    logging::log("SIMULATOR", &format!("Number of Accounts: {}", config.account_config.num_accounts));
            logging::log("SIMULATOR", &format!("Target TPB: {}", config.transaction_config.target_tpb));
    logging::log("SIMULATOR", &format!("Simulation Total Blocks: {}", config.simulation_config.sim_total_block_number));
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