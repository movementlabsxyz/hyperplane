use crate::scenarios::sweep_runner::{SweepRunner, create_modified_config, generate_f64_sequence};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------------
// Sweep-Specific Parameter Struct
// ------------------------------------------------------------------------------------------------

/// Parameters specific to the block interval all scaled sweep simulation.
/// 
/// This struct defines the parameters used to control the block interval all scaled sweep.
/// It contains only the parameters relevant to this specific sweep type.
#[derive(Debug, Deserialize, Clone)]
pub struct BlockIntervalAllScaledSweepParameters {
    /// Total number of simulation runs in the sweep (determines how many parameter values to test)
    pub num_simulations: usize,
    /// Step size for block interval sweeps (in seconds, affects block production rate)
    pub block_interval_step: f64,
    /// Reference TPS for scaling (TPS at 1 second block interval)
    pub reference_tps: f64,
}

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

// Defines the sweep configuration for block interval (all scaled) simulations.
// 
// This macro generates a complete sweep configuration setup including:
// - A config struct with standard fields (network_config, account_config, transaction_config, sweep)
// - Standard validation logic for common fields
// - SweepConfigTrait implementation for integration with the generic SweepRunner
// - A load_config() function that reads and validates the TOML configuration file
define_sweep_config!(
    "sim_sweep_block_interval_all_scaled",
    SweepBlockIntervalAllScaledConfig,
    validate_sweep_specific = |self_: &Self| {
        // Need block_interval_step to generate the sequence of block intervals to test
        if self_.simulation_config.block_interval_step.unwrap_or(0.0) <= 0.0 {
            return Err(crate::config::ConfigError::ValidationError("Block interval step must be positive".into()));
        }
        // Need positive reference_tps for TPS scaling calculation
        if self_.simulation_config.reference_tps.unwrap_or(0.0) <= 0.0 {
            return Err(crate::config::ConfigError::ValidationError("Reference TPS must be positive".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

/// Runs the sweep block interval with all scaled simulation
/// 
/// This simulation explores how different block intervals affect system performance
/// while keeping the delay of the second chain constant and scaling TPS to maintain
/// constant transactions per block.
/// 
/// This simulation scales the TPS inversely with the block interval to maintain
/// a constant number of transactions per block. This approach allows to check if
/// running the simulation with different time intervals (e.g. 0.01s vs 1s) will
/// result in the same simulation results. I note that the results are expected
/// to degrade as the block interval gets too small.
pub async fn run_sweep_block_interval_all_scaled() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_block_interval_all_scaled.toml
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
        "Block Interval (All Scaled)",  // Human-readable name for logging
        "sim_sweep_block_interval_all_scaled",  // Directory name for results
        "block_interval",                  // Parameter name for JSON output
        block_intervals,                   // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, block_interval| {
            create_modified_config(sweep_config, |base_config| {
                // Get the specific sweep config for TPS scaling calculation
                let config = sweep_config.as_any().downcast_ref::<SweepBlockIntervalAllScaledConfig>().unwrap();
                
                // Calculate scaled TPB to maintain constant transactions per block
                // Reference TPB is at 1 second block interval
                // For example, if reference_tpb = 1000:
                // - At 1.0s block interval: TPB = 1000 (reference case)
                // - At 0.5s block interval: TPB = 1000 (same transactions per block)
                // - At 2.0s block interval: TPB = 1000 (same transactions per block)
                let reference_tpb = config.simulation_config.reference_tps.unwrap(); // Still using reference_tps field for backward compatibility
                let target_tpb = reference_tpb; // TPB stays constant regardless of block interval
                
                // Log the configuration for transparency
                crate::logging::log("SIMULATOR", &format!("Block interval: {:.3}s, Target TPB: {:.1} (reference: {:.1} at 1.0s)", 
                    block_interval, target_tpb, reference_tpb));
                
                crate::config::Config {
                    network_config: crate::config::NetworkConfig {
                        num_chains: base_config.network_config.num_chains,
                        chain_delays: vec![
                            base_config.network_config.chain_delays[0],  // Keep first chain delay unchanged
                            base_config.network_config.chain_delays[1],  // Keep second chain delay constant
                        ],
                        block_interval: block_interval,  // Apply the varied block interval
                        channel_buffer_size: base_config.network_config.channel_buffer_size,
                    },
                    account_config: base_config.account_config.clone(),
                    transaction_config: crate::config::TransactionConfig {
                        target_tpb: target_tpb,  // Apply the target TPB
                        zipf_parameter: base_config.transaction_config.zipf_parameter,
                        ratio_cats: base_config.transaction_config.ratio_cats,
                        cat_lifetime_blocks: base_config.transaction_config.cat_lifetime_blocks,
                        allow_cat_pending_dependencies: base_config.transaction_config.allow_cat_pending_dependencies,
                    },
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
/// This function provides the configuration needed to register the block interval all scaled sweep
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepBlockIntervalAllScaled, SimulationConfig {
        name: "Block Interval All Scaled Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_block_interval_all_scaled().await
                .map_err(|e| format!("Block interval all scaled sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_block_interval_all_scaled/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Run with Plotting
// ------------------------------------------------------------------------------------------------

/// Runs the block interval all scaled sweep simulation with automatic plotting.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_block_interval_all_scaled(),
        "Block Interval All Scaled Sweep",
        "simulator/src/scenarios/sim_sweep_block_interval_all_scaled/plot_results.py"
    ).await
} 