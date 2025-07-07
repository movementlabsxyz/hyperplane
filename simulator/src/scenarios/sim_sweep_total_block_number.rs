use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_u64_sequence, SweepConfigTrait};
use crate::config::ValidateConfig;
use std::fs;
use toml;

/// Loads the total block number sweep configuration from the TOML file.
/// 
/// This function reads the configuration file and validates it according to
/// the sweep-specific validation rules.
fn load_config() -> Result<crate::config::SweepDurationConfig, crate::config::ConfigError> {
    let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_total_block_number.toml")?;
    let config: crate::config::SweepDurationConfig = toml::from_str(&config_str)?;
    config.validate()?;
    Ok(config)
}

/// Runs the sweep total block number simulation
/// 
/// This simulation explores how different simulation block counts affect the simulation. 
/// 
/// The sweep varies the total number of blocks to simulate from a minimum to longer periods.
pub async fn run_sweep_total_block_number() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_total_block_number.toml
    let sweep_config = load_config()?;
    
    // Calculate block numbers for each simulation using the helper function
    // Creates a sequence of block numbers: 25, 50, 75, 100, etc.
    // Each value represents the total number of blocks to simulate
    let block_numbers = generate_u64_sequence(
        25,  // Start at 25 blocks
        sweep_config.sweep.duration_step.unwrap(),
        sweep_config.sweep.num_simulations
    );

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Duration",                    // Human-readable name for logging
        "sim_sweep_total_block_number",          // Directory name for results
        "duration",                    // Parameter name for JSON output
        block_numbers,                 // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, block_number| {
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: base_config.network_config.clone(),
                    account_config: base_config.account_config.clone(),
                    transaction_config: crate::config::TransactionConfig {
                        target_tps: base_config.transaction_config.target_tps,
                        sim_total_block_number: block_number,  // This is the parameter we're varying
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
            save_generic_sweep_results(results_dir, "duration", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
}

/// Implementation of SweepConfigTrait for duration sweep configurations.
/// 
/// This allows the SweepRunner to work with configurations specifically designed
/// for simulation duration (total block number) sweeps.
impl SweepConfigTrait for crate::config::SweepDurationConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network_config(&self) -> &crate::config::NetworkConfig { &self.network_config }
    fn get_account_config(&self) -> &crate::config::AccountConfig { &self.account_config }
    fn get_transaction_config(&self) -> &crate::config::TransactionConfig { &self.transaction_config }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Register this simulation with the simulation registry.
/// 
/// This function provides the configuration needed to register the total block number sweep
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepTotalBlockNumber, SimulationConfig {
        name: "Total Block Number Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_total_block_number().await
                .map_err(|e| format!("Total block number sweep failed: {}", e))
        })),
        plot_script: "simulator/scripts/sim_sweep_total_block_number/plot_results.py",
    })
} 