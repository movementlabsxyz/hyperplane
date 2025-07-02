use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

/// Runs the sweep block interval with constant time delay simulation
/// 
/// This simulation explores how different block intervals affect system performance
/// while keeping the delay of the second chain constant at the config value.
/// 
/// The sweep varies the block interval from a minimum to longer periods,
/// running multiple simulations to understand how block production rate affects
/// transaction throughput, success rates, and overall system behavior.
pub async fn run_sweep_block_interval_constant_time_delay() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_block_interval_constant_time_delay.toml
    let sweep_config = crate::config::Config::load_sweep_block_interval_constant_time_delay()?;
    
    // Calculate block intervals for each simulation
    // Creates a sequence of block intervals starting from block_interval_step
    // Each value represents the time between block productions
    let block_intervals: Vec<f64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| sweep_config.sweep.block_interval_step.unwrap() + (i as f64 * sweep_config.sweep.block_interval_step.unwrap()))
        .collect();

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Block Interval (Constant Time Delay)",  // Human-readable name for logging
        "sim_sweep_block_interval_constant_time_delay",  // Directory name for results
        "block_interval",                  // Parameter name for JSON output
        block_intervals,                   // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep_block_interval_constant_time_delay().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        // This takes the base config and applies the current block interval
        // while keeping the second chain delay constant at the config value.
        // Also adjusts the simulation block count to maintain approximately constant simulation time
        Box::new(|sweep_config, block_interval| {
            let config = sweep_config.as_any().downcast_ref::<crate::config::SweepBlockIntervalScaledDelayConfig>().unwrap();
            
            // Calculate delay to maintain constant time delay
            // reference_chain_delay_duration is the delay duration at 0.1 second block interval
            // For any other block interval, we scale it proportionally
            let reference_delay = config.sweep.reference_chain_delay_duration.unwrap_or(0.5);
            let delay_blocks = (reference_delay / 0.1 * block_interval).round() as u64;
            
            // Use the block number from the config
            let block_count = config.transactions.sim_total_block_number;
            
            // Log the configuration for transparency
            crate::logging::log("SIMULATOR", &format!("Block interval: {:.3}s, Block count: {}, Chain 2 delay: {} blocks (reference: {:.1}s at 0.1s)", 
                block_interval, block_count, delay_blocks, reference_delay));
            crate::config::Config {
                network: crate::config::NetworkConfig {
                    num_chains: config.network.num_chains,
                    chain_delays: vec![
                        config.network.chain_delays[0],  // Keep first chain delay unchanged
                        delay_blocks,  // Set second chain delay to maintain constant value
                    ],
                    block_interval: block_interval,                        // Apply the varied block interval
                },
                num_accounts: config.num_accounts.clone(),
                transactions: crate::config::TransactionConfig {
                    target_tps: config.transactions.target_tps,
                    sim_total_block_number: block_count,                    // Use block count from config
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
            save_generic_sweep_results(results_dir, "block_interval", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 