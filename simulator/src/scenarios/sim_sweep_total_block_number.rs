use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

/// Creates a configuration for total block number sweep.
/// 
/// This function takes a sweep configuration and a block number value, then creates
/// a new Config with the total block number applied to the transaction configuration.
/// 
/// # Arguments
/// 
/// * `sweep_config` - The sweep configuration containing base parameters
/// * `block_number` - The total block number value to apply (simulation duration)
/// 
/// # Returns
/// 
/// A new Config with the total block number applied
fn create_total_block_number_config(
    sweep_config: &Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>,
    block_number: u64,
) -> crate::config::Config {
    let config = sweep_config.as_any().downcast_ref::<crate::config::SweepDurationConfig>().unwrap();
    crate::config::Config {
        network_config: config.network_config.clone(),
        account_config: config.account_config.clone(),
        transaction_config: crate::config::TransactionConfig {
            target_tps: config.transaction_config.target_tps,
            sim_total_block_number: block_number,  // This is the parameter we're varying
            zipf_parameter: config.transaction_config.zipf_parameter,
            ratio_cats: config.transaction_config.ratio_cats,
            cat_lifetime_blocks: config.transaction_config.cat_lifetime_blocks,
            initialization_wait_blocks: config.transaction_config.initialization_wait_blocks,
            allow_cat_pending_dependencies: config.transaction_config.allow_cat_pending_dependencies,
        },
    }
}

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
        Box::new(|sweep_config, block_number| {
            create_total_block_number_config(sweep_config, block_number)
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "duration", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 