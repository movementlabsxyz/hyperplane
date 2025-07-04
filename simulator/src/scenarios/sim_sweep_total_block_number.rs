use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

/// Runs the sweep total block number simulation
/// 
/// This simulation explores how different simulation block counts affect the simulation. 
/// 
/// The sweep varies the total number of blocks to simulate from a minimum to longer periods.
pub async fn run_sweep_total_block_number() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_total_block_number.toml
    let sweep_config = crate::config::Config::load_sweep_total_block_number()?;
    
    // Calculate block numbers for each simulation
    // Creates a sequence of block numbers: 25, 50, 75, 100, etc.
    // Each value represents the total number of blocks to simulate
    let block_numbers: Vec<u64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| 25 + (i as u64 * sweep_config.sweep.duration_step.unwrap()))
        .collect();

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Duration",                    // Human-readable name for logging
        "sim_sweep_total_block_number",          // Directory name for results
        "duration",                    // Parameter name for JSON output
        block_numbers,                 // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep_total_block_number().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        // This takes the base config and applies the current block number
        Box::new(|sweep_config, block_number| {
            let config = sweep_config.as_any().downcast_ref::<crate::config::SweepDurationConfig>().unwrap();
            crate::config::Config {
                network: config.network.clone(),
                num_accounts: config.num_accounts.clone(),
                transactions: crate::config::TransactionConfig {
                    target_tps: config.transactions.target_tps,
                    sim_total_block_number: block_number,  // This is the parameter we're varying
                    zipf_parameter: config.transactions.zipf_parameter,
                    ratio_cats: config.transactions.ratio_cats,
                    cat_lifetime_blocks: config.transactions.cat_lifetime_blocks,
                    initialization_wait_blocks: config.transactions.initialization_wait_blocks,
                    allow_cat_pending_dependencies: config.transactions.allow_cat_pending_dependencies,
                },
            }
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "duration", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 