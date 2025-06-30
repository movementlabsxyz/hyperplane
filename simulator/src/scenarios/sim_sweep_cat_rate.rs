use std::env;
use std::fs;
use chrono::Local;
use hyperplane::utils::logging;
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};

/// Runs the sweep CAT rate simulation
pub async fn run_sweep_cat_rate_simulation() -> Result<(), crate::config::ConfigError> {
    // Create results directory if it doesn't exist
    fs::create_dir_all("simulator/results/sim_sweep_cat_rate").expect("Failed to create results directory");
    fs::create_dir_all("simulator/results/sim_sweep_cat_rate/data").expect("Failed to create data directory");
    fs::create_dir_all("simulator/results/sim_sweep_cat_rate/figs").expect("Failed to create figures directory");
    
    // Setup logging
    setup_logging();

    // Load sweep configuration
    let sweep_config = crate::config::Config::load_sweep()?;
    
    // Calculate CAT ratios for each simulation
    let cat_ratios: Vec<f64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| i as f64 * sweep_config.sweep.cat_rate_step)
        .collect();

    logging::log("SIMULATOR", "=== Sweep CAT Rate Simulation ===");
    logging::log("SIMULATOR", &format!("Number of simulations: {}", sweep_config.sweep.num_simulations));
    logging::log("SIMULATOR", &format!("CAT rate step: {}", sweep_config.sweep.cat_rate_step));
    logging::log("SIMULATOR", &format!("CAT ratios: {:?}", cat_ratios));
    logging::log("SIMULATOR", "================================");

    // Create progress bar for sweep
    let progress_bar = ProgressBar::new(sweep_config.sweep.num_simulations as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
        .unwrap()
        .progress_chars("+>-"));

    // Store results for each simulation
    let mut all_results = Vec::new();

    // Run each simulation with different CAT ratio
    for (sim_index, cat_ratio) in cat_ratios.iter().enumerate() {
        logging::log("SIMULATOR", &format!("Running simulation {}/{} with CAT ratio: {:.3}", 
            sim_index + 1, sweep_config.sweep.num_simulations, cat_ratio));

        // Create a modified config with the current CAT ratio
        let sim_config = crate::config::Config {
            network: sweep_config.network.clone(),
            num_accounts: sweep_config.num_accounts.clone(),
            transactions: crate::config::TransactionConfig {
                target_tps: sweep_config.transactions.target_tps,
                duration_seconds: sweep_config.transactions.duration_seconds,
                zipf_parameter: sweep_config.transactions.zipf_parameter,
                ratio_cats: *cat_ratio,
                cat_lifetime_blocks: sweep_config.transactions.cat_lifetime_blocks,
            },
        };

        // Initialize simulation results
        let mut results = initialize_simulation_results(&sim_config, sim_index, *cat_ratio);

        // Setup test nodes
        let (_hs_node, cl_node, hig_node_1, hig_node_2, _start_block_height) = crate::testnodes::setup_test_nodes(
            Duration::from_secs_f64(sim_config.network.block_interval),
            &sim_config.network.chain_delays,
        ).await;
        
        // Initialize accounts with initial balance
        crate::network::initialize_accounts(
            &[cl_node.clone()], 
            sim_config.num_accounts.initial_balance.try_into().unwrap(), 
            sim_config.num_accounts.num_accounts.try_into().unwrap()
        ).await;

        // Run simulation
        crate::run_simulation::run_simulation(
            cl_node,
            vec![hig_node_1, hig_node_2],
            &mut results,
        ).await.map_err(|e| crate::config::ConfigError::ValidationError(e))?;

        // Save individual simulation results
        results.save_to_directory(&format!("simulator/results/sim_sweep_cat_rate/data/sim_{}", sim_index)).await.map_err(|e| crate::config::ConfigError::ValidationError(e))?;
        
        all_results.push((*cat_ratio, results));
        
        // Update progress bar and show completed simulation
        progress_bar.inc(1);
        progress_bar.set_message(format!("Simulation {}/{} with CAT ratio: {:.3}", 
            sim_index + 1, sweep_config.sweep.num_simulations, cat_ratio));
    }

    // Finish progress bar with final state
    progress_bar.finish_with_message(format!("Simulation {}/{} with CAT ratio: {:.3}", 
        sweep_config.sweep.num_simulations, sweep_config.sweep.num_simulations, 
        cat_ratios.last().unwrap()));
    
    println!("Sweep simulation complete");

    // Save combined results
    save_sweep_results(&all_results).await?;

    logging::log("SIMULATOR", "=== Sweep Simulation Complete ===");
    logging::log("SIMULATOR", &format!("Total simulations completed: {}", all_results.len()));

    Ok(())
}

/// Sets up logging if ENABLE_LOGS environment variable is set
fn setup_logging() {
    if env::var("ENABLE_LOGS").is_ok() {
        // Delete existing log file if it exists
        let log_path = "simulator/results/sim_sweep_cat_rate/simulation.log";
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
fn initialize_simulation_results(config: &crate::config::Config, sim_index: usize, cat_ratio: f64) -> crate::SimulationResults {
    let mut results = crate::SimulationResults::default();
    results.initial_balance = config.num_accounts.initial_balance.try_into().unwrap();
    results.num_accounts = config.num_accounts.num_accounts.try_into().unwrap();
    results.target_tps = config.transactions.target_tps as u64;
    results.duration_seconds = config.transactions.duration_seconds.try_into().unwrap();
    results.zipf_parameter = config.transactions.zipf_parameter;
    results.ratio_cats = config.transactions.ratio_cats;
    results.block_interval = config.network.block_interval;
    results.cat_lifetime = config.transactions.cat_lifetime_blocks;
    results.chain_delays = config.network.chain_delays.clone();
    results.start_time = Instant::now();

    // Log configuration
    let start_time = Local::now();
    logging::log("SIMULATOR", &format!("=== Simulation {} Configuration ===", sim_index + 1));
    logging::log("SIMULATOR", &format!("Start Time: {}", start_time.format("%Y-%m-%d %H:%M:%S")));
    logging::log("SIMULATOR", &format!("CAT Ratio: {:.3}", cat_ratio));
    logging::log("SIMULATOR", &format!("Initial Balance: {}", config.num_accounts.initial_balance));
    logging::log("SIMULATOR", &format!("Number of Accounts: {}", config.num_accounts.num_accounts));
    logging::log("SIMULATOR", &format!("Target TPS: {}", config.transactions.target_tps));
    logging::log("SIMULATOR", &format!("Simulation Duration: {} seconds", config.transactions.duration_seconds));
    logging::log("SIMULATOR", &format!("Number of Chains: {}", config.network.num_chains));
    logging::log("SIMULATOR", &format!("Zipf Parameter: {}", config.transactions.zipf_parameter));
    logging::log("SIMULATOR", &format!("CAT Lifetime: {} blocks", results.cat_lifetime));
    for (i, delay) in config.network.chain_delays.iter().enumerate() {
        logging::log("SIMULATOR", &format!("Chain {} Delay: {:?}", i + 1, delay));
    }
    logging::log("SIMULATOR", "=============================");

    results
}

/// Saves combined sweep results
async fn save_sweep_results(all_results: &[(f64, crate::SimulationResults)]) -> Result<(), crate::config::ConfigError> {
    use serde_json;
    
    // Create combined results structure
    let combined_results = serde_json::json!({
        "sweep_summary": {
            "num_simulations": all_results.len(),
            "cat_ratios": all_results.iter().map(|(ratio, _)| ratio).collect::<Vec<_>>(),
            "total_transactions": all_results.iter().map(|(_, results)| results.transactions_sent).collect::<Vec<_>>(),
            "cat_transactions": all_results.iter().map(|(_, results)| results.cat_transactions).collect::<Vec<_>>(),
            "regular_transactions": all_results.iter().map(|(_, results)| results.regular_transactions).collect::<Vec<_>>(),
        },
        "individual_results": all_results.iter().map(|(ratio, results)| {
            serde_json::json!({
                "cat_ratio": ratio,
                "total_transactions": results.transactions_sent,
                "cat_transactions": results.cat_transactions,
                "regular_transactions": results.regular_transactions,
                "chain_1_pending": results.chain_1_pending,
                "chain_1_success": results.chain_1_success,
                "chain_1_failure": results.chain_1_failure,
            })
        }).collect::<Vec<_>>()
    });

    // Save combined results
    let combined_file = "simulator/results/sim_sweep_cat_rate/data/sweep_results.json";
    fs::write(combined_file, serde_json::to_string_pretty(&combined_results).expect("Failed to serialize combined results"))
        .map_err(|e| crate::config::ConfigError::ValidationError(e.to_string()))?;
    
    logging::log("SIMULATOR", &format!("Saved combined sweep results to {}", combined_file));

    Ok(())
} 