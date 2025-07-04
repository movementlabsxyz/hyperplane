use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

/// Runs the sweep chain delay simulation
/// 
/// This simulation explores how a HIG delay to the HS affects
/// system performance.
/// 
/// The sweep varies the delay of the second chain (HIG to HS),
/// running multiple simulations to understand how it affects
/// transaction throughput, success rates, and overall system behavior.
pub async fn run_sweep_chain_delay() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_chain_delay.toml
    let sweep_config = crate::config::Config::load_sweep_chain_delay()?;
    
    // Calculate chain delays for each simulation
    // Creates a sequence of delays: 0 blocks, 1 block, 2 blocks, 3 blocks, etc.
    // Each value represents the delay from the HIG to HS in blocks
    let chain_delays: Vec<u64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| i as u64 * sweep_config.sweep.chain_delay_step.unwrap() as u64)
        .collect();

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Chain Delay",                 // Human-readable name for logging
        "sim_sweep_chain_delay",       // Directory name for results
        "chain_delay",                 // Parameter name for JSON output
        chain_delays,                  // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep_chain_delay().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        // This takes the base config and applies the current delay from HIG to HS
        Box::new(|sweep_config, chain_delay| {
            let config = sweep_config.as_any().downcast_ref::<crate::config::SweepChainDelayConfig>().unwrap();
            crate::config::Config {
                network: crate::config::NetworkConfig {
                    num_chains: config.network.num_chains,
                    chain_delays: vec![
                        config.network.chain_delays[0],  // Keep first chain delay unchanged
                        chain_delay,                     // Apply delay to second chain in blocks
                    ],
                    block_interval: config.network.block_interval,
                },
                num_accounts: config.num_accounts.clone(),
                transactions: config.transactions.clone(),
            }
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "chain_delay", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 