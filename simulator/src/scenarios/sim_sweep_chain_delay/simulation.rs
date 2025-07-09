use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_u64_sequence};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------------
// Sweep-Specific Parameter Struct
// ------------------------------------------------------------------------------------------------

/// Parameters specific to the chain delay sweep simulation.
/// 
/// This struct defines the parameters used to control the chain delay sweep.
/// It contains only the parameters relevant to this specific sweep type.
#[derive(Debug, Deserialize, Clone)]
pub struct ChainDelaySweepParameters {
    /// Total number of simulation runs in the sweep (determines how many parameter values to test)
    pub num_simulations: usize,
    /// Step size for chain delay sweeps (in blocks, affects inter-chain communication timing)
    pub chain_delay_step: f64,
}

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

// Defines the sweep configuration for chain delay simulations.
// 
// This macro generates a complete sweep configuration setup including:
// - A config struct with standard fields (network_config, account_config, transaction_config, sweep)
// - Standard validation logic for common fields
// - SweepConfigTrait implementation for integration with the generic SweepRunner
// - A load_config() function that reads and validates the TOML configuration file
define_sweep_config!(
    "sim_sweep_chain_delay",
    SweepChainDelayConfig,
    sweep_parameters = ChainDelaySweepParameters,
    validate_sweep_specific = |self_: &Self| {
        // Need chain_delay_step to generate the sequence of chain delays to test
        if self_.sweep.chain_delay_step <= 0.0 {
            return Err(crate::config::ConfigError::ValidationError("Chain delay step must be positive".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

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
    let sweep_config = load_config()?;
    
    // Calculate chain delays for each simulation using the helper function
    // Creates a sequence of delays: 0 blocks, 1 block, 2 blocks, 3 blocks, etc.
    // Each value represents the delay from the HIG to HS in blocks
    let chain_delays = generate_u64_sequence(
        0,  // Start at 0 blocks
        sweep_config.sweep.chain_delay_step as u64,
        sweep_config.sweep.num_simulations
    );

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Chain Delay",                 // Human-readable name for logging
        "sim_sweep_chain_delay",       // Directory name for results
        "chain_delay",                 // Parameter name for JSON output
        chain_delays,                  // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, chain_delay| {
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: crate::config::NetworkConfig {
                        num_chains: base_config.network_config.num_chains,
                        chain_delays: vec![
                            base_config.network_config.chain_delays[0],  // Keep first chain delay unchanged
                            chain_delay,                     // Apply delay to second chain in blocks
                        ],
                        block_interval: base_config.network_config.block_interval,
                    },
                    account_config: base_config.account_config.clone(),
                    transaction_config: base_config.transaction_config.clone(),
                }
            })
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "chain_delay", all_results)
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
/// This function provides the configuration needed to register the chain delay sweep
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepChainDelay, SimulationConfig {
        name: "Chain Delay Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_chain_delay().await
                .map_err(|e| format!("Chain delay sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_chain_delay/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Run with Plotting
// ------------------------------------------------------------------------------------------------

/// Runs the chain delay sweep simulation with automatic plotting.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_chain_delay(),
        "Chain Delay Sweep",
        "simulator/src/scenarios/sim_sweep_chain_delay/plot_results.py"
    ).await
} 