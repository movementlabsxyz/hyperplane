use crate::scenarios::sweep_runner::{SweepRunner, create_modified_config, generate_f64_sequence};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------------
// Sweep-Specific Parameter Struct
// ------------------------------------------------------------------------------------------------

/// Parameters specific to the block interval constant block delay sweep simulation.
/// 
/// This struct defines the parameters used to control the block interval constant block delay sweep.
/// It contains only the parameters relevant to this specific sweep type.
#[derive(Debug, Deserialize, Clone)]
pub struct BlockIntervalConstantBlockDelaySweepParameters {
    /// Total number of simulation runs in the sweep (determines how many parameter values to test)
    pub num_simulations: usize,
    /// Step size for block interval sweeps (in seconds, affects block production rate)
    pub block_interval_step: f64,
}

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

// Defines the sweep configuration for block interval (constant block delay) simulations.
// 
// This macro generates a complete sweep configuration setup including:
// - A config struct with standard fields (network_config, account_config, transaction_config, sweep)
// - Standard validation logic for common fields
// - SweepConfigTrait implementation for integration with the generic SweepRunner
// - A load_config() function that reads and validates the TOML configuration file
define_sweep_config!(
    "sim_sweep_block_interval_constant_block_delay",
    SweepBlockIntervalConstantBlockDelayConfig,
    validate_sweep_specific = |self_: &Self| {
        // Need block_interval_step to generate the sequence of block intervals to test
        if self_.simulation_config.block_interval_step.unwrap_or(0.0) <= 0.0 {
            return Err(crate::config::ConfigError::ValidationError("Block interval step must be positive".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

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
        sweep_config.simulation_config.block_interval_step.unwrap(),
        sweep_config.simulation_config.block_interval_step.unwrap(),
        sweep_config.simulation_config.num_simulations.unwrap()
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
                        channel_buffer_size: base_config.network_config.channel_buffer_size,
                    },
                    account_config: base_config.account_config.clone(),
                    transaction_config: base_config.transaction_config.clone(),
                    simulation_config: base_config.simulation_config.clone(),
                    logging_config: base_config.logging_config.clone(),
                }
            })
        }),
        // Function to save the combined results from all simulations
        // Note: Data is now handled by the averaging script and plotting code
        Box::new(|_results_dir, _all_results| {
            Ok(())
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
}

// ------------------------------------------------------------------------------------------------
// Simulation Registration
// ------------------------------------------------------------------------------------------------

/// Register this simulation with the simulation registry.
/// 
/// This function provides the configuration needed to register the block interval constant block delay sweep
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepBlockIntervalConstantBlockDelay, SimulationConfig {
        name: "Block Interval Constant Block Delay Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_block_interval_constant_block_delay().await
                .map_err(|e| format!("Block interval constant block delay sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_block_interval_constant_block_delay/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Run with Plotting
// ------------------------------------------------------------------------------------------------

/// Runs the block interval constant block delay sweep simulation with automatic plotting.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_block_interval_constant_block_delay(),
        "Block Interval Constant Block Delay Sweep",
        "simulator/src/scenarios/sim_sweep_block_interval_constant_block_delay/plot_results.py"
    ).await
} 