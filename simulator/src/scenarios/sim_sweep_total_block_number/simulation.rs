use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_u64_sequence};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------------
// Sweep-Specific Parameter Struct
// ------------------------------------------------------------------------------------------------

/// Parameters specific to the total block number sweep simulation.
/// 
/// This struct defines the parameters used to control the total block number sweep.
/// It contains only the parameters relevant to this specific sweep type.
#[derive(Debug, Deserialize, Clone)]
pub struct TotalBlockNumberSweepParameters {
    /// Total number of simulation runs in the sweep (determines how many parameter values to test)
    pub num_simulations: usize,
    /// Step size for total block number sweeps (in blocks, affects total simulation length)
    pub block_number_step: u64,
}

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

// Defines the sweep configuration for total block number simulations.
// 
// This macro generates a complete sweep configuration setup including:
// - A config struct with standard fields (network_config, account_config, transaction_config, sweep)
// - Standard validation logic for common fields
// - SweepConfigTrait implementation for integration with the generic SweepRunner
// - A load_config() function that reads and validates the TOML configuration file
define_sweep_config!(
    "sim_sweep_total_block_number",
    SweepTotalBlockNumberConfig,
    validate_sweep_specific = |self_: &Self| {
        // Need block_number_step to generate the sequence of block counts to test
        if self_.simulation_config.block_number_step.unwrap_or(0) == 0 {
            return Err(crate::config::ConfigError::ValidationError("Block number step must be positive".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

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
    // Creates a sequence starting from block_number_step and stepping by block_number_step
    // Each value represents the total number of blocks to simulate
    let step_size = sweep_config.simulation_config.block_number_step.unwrap();
    let num_sims = sweep_config.simulation_config.num_simulations.unwrap();
    
    let block_numbers = generate_u64_sequence(
        step_size,          // Start at the step size (e.g., 250 blocks)
        step_size,          // Step by the configured step size
        num_sims            // Number of simulations from config
    );

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Total Block Number",          // Human-readable name for logging
        "sim_sweep_total_block_number",          // Directory name for results
        "total_block_number",          // Parameter name for JSON output
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
                        zipf_parameter: base_config.transaction_config.zipf_parameter,
                        ratio_cats: base_config.transaction_config.ratio_cats,
                        cat_lifetime_blocks: base_config.transaction_config.cat_lifetime_blocks,
                        allow_cat_pending_dependencies: base_config.transaction_config.allow_cat_pending_dependencies,
                    },
                    simulation_config: crate::config::SimulationConfig {
                        sim_total_block_number: block_number,  // This is the parameter we're varying
                        ..base_config.simulation_config.clone()
                    },
                    logging_config: base_config.logging_config.clone(),
                }
            })
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "total_block_number", all_results)
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
        plot_script: "simulator/src/scenarios/sim_sweep_total_block_number/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Run with Plotting
// ------------------------------------------------------------------------------------------------

/// Runs the total block number sweep simulation with automatic plotting.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_total_block_number(),
        "Total Block Number Sweep",
        "simulator/src/scenarios/sim_sweep_total_block_number/plot_results.py"
    ).await
} 