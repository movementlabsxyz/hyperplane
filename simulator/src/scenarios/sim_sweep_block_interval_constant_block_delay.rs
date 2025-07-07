use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_f64_sequence, SweepConfigTrait};
use crate::config::ValidateConfig;
use std::fs;
use toml;

/// Loads the block interval constant block delay sweep configuration from the TOML file.
/// 
/// This function reads the configuration file and validates it according to
/// the sweep-specific validation rules.
fn load_config() -> Result<crate::config::SweepBlockIntervalConstantDelayConfig, crate::config::ConfigError> {
    let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_block_interval_constant_block_delay.toml")?;
    let config: crate::config::SweepBlockIntervalConstantDelayConfig = toml::from_str(&config_str)?;
    config.validate()?;
    Ok(config)
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
        "Block Interval (Constant Block Delay)",  // Human-readable name for logging
        "sim_sweep_block_interval_constant_block_delay",  // Directory name for results
        "block_interval",                  // Parameter name for JSON output
        block_intervals,                   // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, block_interval| {
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: crate::config::NetworkConfig {
                        num_chains: base_config.network_config.num_chains,
                        chain_delays: vec![
                            base_config.network_config.chain_delays[0],  // Keep first chain delay unchanged
                            base_config.network_config.chain_delays[1],  // Use the second chain delay for constant block delay
                        ],
                        block_interval: block_interval,                        // Apply the varied block interval
                    },
                    account_config: base_config.account_config.clone(),
                    transaction_config: base_config.transaction_config.clone(),
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

/// Implementation of SweepConfigTrait for block interval constant delay sweep configurations.
/// 
/// This allows the SweepRunner to work with configurations specifically designed
/// for block interval sweeps with constant block delays.
impl SweepConfigTrait for crate::config::SweepBlockIntervalConstantDelayConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network_config(&self) -> &crate::config::NetworkConfig { &self.network_config }
    fn get_account_config(&self) -> &crate::config::AccountConfig { &self.account_config }
    fn get_transaction_config(&self) -> &crate::config::TransactionConfig { &self.transaction_config }
    fn as_any(&self) -> &dyn std::any::Any { self }
} 