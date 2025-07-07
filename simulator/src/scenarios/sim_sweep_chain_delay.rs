
use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_u64_sequence};
use crate::config::ValidateConfig;
use std::fs;
use toml;

// ------------------------------------------------------------------------------------------------
// Configuration Loading
// ------------------------------------------------------------------------------------------------

/// Loads the chain delay sweep configuration from the TOML file.
/// 
/// This function reads the configuration file and validates it according to
/// the sweep-specific validation rules.
fn load_config() -> Result<crate::config::SweepChainDelayConfig, crate::config::ConfigError> {
    let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_chain_delay.toml")?;
    let config: crate::config::SweepChainDelayConfig = toml::from_str(&config_str)?;
    config.validate()?;
    Ok(config)
}

// ------------------------------------------------------------------------------------------------
// Parameter Sequence Generation & Sweep Runner Setup
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
        sweep_config.sweep.chain_delay_step.unwrap() as u64,
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
        plot_script: "simulator/scripts/sim_sweep_chain_delay/plot_results.py",
    })
} 