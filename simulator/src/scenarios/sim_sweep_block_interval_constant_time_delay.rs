use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_f64_sequence, SweepConfigTrait};
use crate::config::ValidateConfig;
use std::fs;
use toml;

/// Loads the block interval constant time delay sweep configuration from the TOML file.
/// 
/// This function reads the configuration file and validates it according to
/// the sweep-specific validation rules.
fn load_config() -> Result<crate::config::SweepBlockIntervalScaledDelayConfig, crate::config::ConfigError> {
    let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_block_interval_constant_time_delay.toml")?;
    let config: crate::config::SweepBlockIntervalScaledDelayConfig = toml::from_str(&config_str)?;
    config.validate()?;
    Ok(config)
}

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
    let sweep_config = load_config()?;
    
    // Calculate block intervals for each simulation using the helper function
    // Creates a sequence of block intervals starting from block_interval_step
    // Each value represents the time between block productions
    let block_intervals = generate_f64_sequence(
        sweep_config.sweep.block_interval_step.unwrap(),
        sweep_config.sweep.block_interval_step.unwrap(),
        sweep_config.sweep.num_simulations
    );

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Block Interval (Constant Time Delay)",  // Human-readable name for logging
        "sim_sweep_block_interval_constant_time_delay",  // Directory name for results
        "block_interval",                  // Parameter name for JSON output
        block_intervals,                   // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, block_interval| {
            create_modified_config(sweep_config, |base_config| {
                // Get the specific sweep config for delay calculation
                let config = sweep_config.as_any().downcast_ref::<crate::config::SweepBlockIntervalScaledDelayConfig>().unwrap();
                
                // Calculate delay to maintain constant time delay
                // reference_chain_delay_duration is the time delay in seconds
                // For example, if reference_delay = 0.5 seconds:
                // At 0.1s block interval, this requires 5 blocks = 0.5 seconds
                // At 0.05s block interval, this requires 10 blocks = 0.5 seconds
                let reference_delay = config.sweep.reference_chain_delay_duration.unwrap_or(0.5);
                let delay_blocks = (reference_delay / block_interval).round() as u64;
                
                // Use the block number from the config
                let block_count = config.transaction_config.sim_total_block_number;
                
                // Log the configuration for transparency
                crate::logging::log("SIMULATOR", &format!("Block interval: {:.3}s, Block count: {}, Chain 2 delay: {} blocks (reference: {:.1}s at 0.1s)", 
                    block_interval, block_count, delay_blocks, reference_delay));
                
                crate::config::Config {
                    network_config: crate::config::NetworkConfig {
                        num_chains: base_config.network_config.num_chains,
                        chain_delays: vec![
                            base_config.network_config.chain_delays[0],  // Keep first chain delay unchanged
                            delay_blocks,  // Set second chain delay to maintain constant value
                        ],
                        block_interval: block_interval,  // Apply the varied block interval
                    },
                    account_config: base_config.account_config.clone(),
                    transaction_config: crate::config::TransactionConfig {
                        target_tps: base_config.transaction_config.target_tps,
                        sim_total_block_number: block_count,  // Use block count from config
                        zipf_parameter: base_config.transaction_config.zipf_parameter,
                        ratio_cats: base_config.transaction_config.ratio_cats,
                        cat_lifetime_blocks: base_config.transaction_config.cat_lifetime_blocks,
                        initialization_wait_blocks: base_config.transaction_config.initialization_wait_blocks,
                        allow_cat_pending_dependencies: base_config.transaction_config.allow_cat_pending_dependencies,
                    },
                }
            })
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "block_interval", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
}

/// Implementation of SweepConfigTrait for block interval scaled delay sweep configurations.
/// 
/// This allows the SweepRunner to work with configurations specifically designed
/// for block interval sweeps with scaled time delays.
impl SweepConfigTrait for crate::config::SweepBlockIntervalScaledDelayConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network_config(&self) -> &crate::config::NetworkConfig { &self.network_config }
    fn get_account_config(&self) -> &crate::config::AccountConfig { &self.account_config }
    fn get_transaction_config(&self) -> &crate::config::TransactionConfig { &self.transaction_config }
    fn as_any(&self) -> &dyn std::any::Any { self }
} 