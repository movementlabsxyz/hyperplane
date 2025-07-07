use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_f64_sequence};
use crate::define_sweep_config;
use crate::config::ValidateConfig;

// ============================================================================
// Sweep Configuration
// ============================================================================

define_sweep_config!(
    SweepBlockIntervalConstantDelayConfig,
    "config_sweep_block_interval_constant_block_delay.toml",
    validate_sweep_specific = |self_: &Self| {
        // Need block_interval_step to generate the sequence of block intervals to test
        if self_.sweep.block_interval_step.is_none() {
            return Err(crate::config::ConfigError::ValidationError("Block interval step must be specified".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Parameter Sequence Generation & Sweep Runner Setup
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
        plot_script: "simulator/scripts/sim_sweep_block_interval_constant_block_delay/plot_results.py",
    })
} 