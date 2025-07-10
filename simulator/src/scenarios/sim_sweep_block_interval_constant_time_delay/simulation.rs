use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_f64_sequence};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------------
// Sweep-Specific Parameter Struct
// ------------------------------------------------------------------------------------------------

/// Parameters specific to the block interval constant time delay sweep simulation.
/// 
/// This struct defines the parameters used to control the block interval constant time delay sweep.
/// It contains only the parameters relevant to this specific sweep type.
#[derive(Debug, Deserialize, Clone)]
pub struct BlockIntervalConstantTimeDelaySweepParameters {
    /// Total number of simulation runs in the sweep (determines how many parameter values to test)
    pub num_simulations: usize,
    /// Step size for block interval sweeps (in seconds, affects block production rate)
    pub block_interval_step: f64,
    /// Reference delay duration for block interval sweeps (in seconds, used with block_interval_step)
    pub reference_chain_delay_duration: f64,
}

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

// Defines the sweep configuration for block interval (constant time delay) simulations.
// 
// This macro generates a complete sweep configuration setup including:
// - A config struct with standard fields (network_config, account_config, transaction_config, sweep)
// - Standard validation logic for common fields
// - SweepConfigTrait implementation for integration with the generic SweepRunner
// - A load_config() function that reads and validates the TOML configuration file
define_sweep_config!(
    "sim_sweep_block_interval_constant_time_delay",
    SweepBlockIntervalScaledDelayConfig,
    sweep_parameters = BlockIntervalConstantTimeDelaySweepParameters,
    validate_sweep_specific = |self_: &Self| {
        // Need block_interval_step to generate the sequence of block intervals to test
        if self_.sweep.block_interval_step <= 0.0 {
            return Err(crate::config::ConfigError::ValidationError("Block interval step must be positive".into()));
        }
        // Need positive reference_delay for delay calculation: delay_blocks = reference_delay / block_interval
        if self_.sweep.reference_chain_delay_duration <= 0.0 {
            return Err(crate::config::ConfigError::ValidationError("Reference chain delay duration must be positive".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

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
        sweep_config.sweep.block_interval_step,
        sweep_config.sweep.block_interval_step,
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
                let config = sweep_config.as_any().downcast_ref::<SweepBlockIntervalScaledDelayConfig>().unwrap();
                
                // Calculate delay to maintain constant time delay
                // reference_chain_delay_duration is the time delay in seconds
                // For example, if reference_delay = 0.5 seconds:
                // At 0.1s block interval, this requires 5 blocks = 0.5 seconds
                // At 0.05s block interval, this requires 10 blocks = 0.5 seconds
                let reference_delay = config.sweep.reference_chain_delay_duration;
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
                        funding_wait_blocks: base_config.transaction_config.funding_wait_blocks,
                        allow_cat_pending_dependencies: base_config.transaction_config.allow_cat_pending_dependencies,
                    },
                    repeat_config: base_config.repeat_config.clone(),
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
/// This function provides the configuration needed to register the block interval constant time delay sweep
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepBlockIntervalConstantTimeDelay, SimulationConfig {
        name: "Block Interval Constant Time Delay Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_block_interval_constant_time_delay().await
                .map_err(|e| format!("Block interval constant time delay sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_block_interval_constant_time_delay/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Run with Plotting
// ------------------------------------------------------------------------------------------------

/// Runs the block interval constant time delay sweep simulation with automatic plotting.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_block_interval_constant_time_delay(),
        "Block Interval Constant Time Delay Sweep",
        "simulator/src/scenarios/sim_sweep_block_interval_constant_time_delay/plot_results.py"
    ).await
} 