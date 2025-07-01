use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

/// Runs the sweep block interval with constant delay simulation
/// 
/// This simulation explores how different block intervals affect system performance
/// while keeping the delay of the second chain constant at 0.5 seconds.
/// 
/// The sweep varies the block interval from a minimum to longer periods,
/// running multiple simulations to understand how block production rate affects
/// transaction throughput, success rates, and overall system behavior.
pub async fn run_sweep_block_interval_constant_delay() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_block_interval_constant_delay.toml
    let sweep_config = crate::config::Config::load_sweep_block_interval_constant_delay()?;
    
    // Calculate block intervals for each simulation
    // Creates a sequence of block intervals starting from block_interval_step
    // Each value represents the time between block productions
    let block_intervals: Vec<f64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| sweep_config.sweep.block_interval_step.unwrap() + (i as f64 * sweep_config.sweep.block_interval_step.unwrap()))
        .collect();

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Block Interval (Constant Delay)",  // Human-readable name for logging
        "sim_sweep_block_interval_constant_delay",  // Directory name for results
        "block_interval",                  // Parameter name for JSON output
        block_intervals,                   // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep_block_interval_constant_delay().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        // This takes the base config and applies the current block interval
        // while keeping the second chain delay constant at 0.5 seconds
        // Also adjusts the simulation block count to maintain approximately constant simulation time
        Box::new(|sweep_config, block_interval| {
            let config = sweep_config.as_any().downcast_ref::<crate::config::SweepBlockIntervalConstantDelayConfig>().unwrap();
            
            // Calculate target simulation time (approximately 5 seconds)
            let target_simulation_time = 5.0; // seconds
            let adjusted_block_count = (target_simulation_time / block_interval).round() as u64;
            
            // Log the adjustment for transparency
            crate::logging::log("SIMULATOR", &format!("Block interval: {:.3}s, Adjusted block count: {} (target time: {:.1}s)", 
                block_interval, adjusted_block_count, target_simulation_time));
            
            crate::config::Config {
                network: crate::config::NetworkConfig {
                    num_chains: config.network.num_chains,
                    chain_delays: vec![
                        config.network.chain_delays[0],                    // Keep first chain delay unchanged
                        config.network.chain_delays[1],                    // Keep second chain delay constant at 0.5s
                    ],
                    block_interval: block_interval,                        // Apply the varied block interval
                },
                num_accounts: config.num_accounts.clone(),
                transactions: crate::config::TransactionConfig {
                    target_tps: config.transactions.target_tps,
                    sim_total_block_number: adjusted_block_count,          // Adjust block count to maintain constant time
                    zipf_parameter: config.transactions.zipf_parameter,
                    ratio_cats: config.transactions.ratio_cats,
                    cat_lifetime_blocks: config.transactions.cat_lifetime_blocks,
                    initialization_wait_blocks: config.transactions.initialization_wait_blocks,
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