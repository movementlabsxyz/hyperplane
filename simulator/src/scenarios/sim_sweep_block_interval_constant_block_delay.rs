use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

/// Creates a configuration for block interval constant block delay sweep.
/// 
/// This function takes a sweep configuration and a block interval value, then creates
/// a new Config with the block interval applied to the network configuration while
/// keeping the second chain delay constant.
/// 
/// # Arguments
/// 
/// * `sweep_config` - The sweep configuration containing base parameters
/// * `block_interval` - The block interval value to apply (time between blocks)
/// 
/// # Returns
/// 
/// A new Config with the block interval applied
fn create_block_interval_constant_block_delay_config(
    sweep_config: &Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>,
    block_interval: f64,
) -> crate::config::Config {
    let config = sweep_config.as_any().downcast_ref::<crate::config::SweepBlockIntervalConstantDelayConfig>().unwrap();

    crate::config::Config {
        network_config: crate::config::NetworkConfig {
            num_chains: config.network_config.num_chains,
            chain_delays: vec![
                config.network_config.chain_delays[0],  // Keep first chain delay unchanged
                config.network_config.chain_delays[1],  // Use the second chain delay for constant block delay
            ],
            block_interval: block_interval,                        // Apply the varied block interval
        },
        account_config: config.account_config.clone(),
        transaction_config: crate::config::TransactionConfig {
            target_tps: config.transaction_config.target_tps,
            sim_total_block_number: config.transaction_config.sim_total_block_number,                    
            zipf_parameter: config.transaction_config.zipf_parameter,
            ratio_cats: config.transaction_config.ratio_cats,
            cat_lifetime_blocks: config.transaction_config.cat_lifetime_blocks,
            initialization_wait_blocks: config.transaction_config.initialization_wait_blocks,
            allow_cat_pending_dependencies: config.transaction_config.allow_cat_pending_dependencies,
        },
    }
}

/// Runs the sweep block interval with constant block delay simulation
/// 
/// This simulation explores how different block intervals affect system performance
/// while keeping the delay of the second chain constant at the config value.
/// 
/// The sweep varies the block interval from a minimum to longer periods,
/// running multiple simulations to understand how block production rate affects
/// transaction throughput, success rates, and overall system behavior.
pub async fn run_sweep_block_interval_constant_block_delay() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_block_interval_constant_block_delay.toml
    let sweep_config = crate::config::Config::load_sweep_block_interval_constant_block_delay()?;
    
    // Calculate block intervals for each simulation
    // Creates a sequence of block intervals starting from block_interval_step
    // Each value represents the time between block productions
    let block_intervals: Vec<f64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| sweep_config.sweep.block_interval_step.unwrap() + (i as f64 * sweep_config.sweep.block_interval_step.unwrap()))
        .collect();

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Block Interval (Constant Block Delay)",  // Human-readable name for logging
        "sim_sweep_block_interval_constant_block_delay",  // Directory name for results
        "block_interval",                  // Parameter name for JSON output
        block_intervals,                   // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep_block_interval_constant_block_delay().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        Box::new(|sweep_config, block_interval| {
            create_block_interval_constant_block_delay_config(sweep_config, block_interval)
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "block_interval", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 