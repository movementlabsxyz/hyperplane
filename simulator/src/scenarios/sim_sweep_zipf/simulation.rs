use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_f64_sequence};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------------
// Sweep-Specific Parameter Struct
// ------------------------------------------------------------------------------------------------

/// Parameters specific to the Zipf distribution sweep simulation.
/// 
/// This struct defines the parameters used to control the Zipf distribution sweep.
/// It contains only the parameters relevant to this specific sweep type.
#[derive(Debug, Deserialize, Clone)]
pub struct ZipfSweepParameters {
    /// Total number of simulation runs in the sweep (determines how many parameter values to test)
    pub num_simulations: usize,
    /// Step size for Zipf distribution parameter sweeps (controls account access pattern skewness)
    pub zipf_step: f64,
}

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

// Defines the sweep configuration for Zipf distribution simulations.
// 
// This macro generates a complete sweep configuration setup including:
// - A config struct with standard fields (network_config, account_config, transaction_config, sweep)
// - Standard validation logic for common fields
// - SweepConfigTrait implementation for integration with the generic SweepRunner
// - A load_config() function that reads and validates the TOML configuration file
define_sweep_config!(
    "sim_sweep_zipf",
    SweepZipfConfig,
    sweep_parameters = ZipfSweepParameters,
    validate_sweep_specific = |self_: &Self| {
        // Need zipf_step to generate the sequence of Zipf parameters to test
        if self_.sweep.zipf_step <= 0.0 {
            return Err(crate::config::ConfigError::ValidationError("Zipf step must be positive".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

/// Runs the sweep Zipf distribution simulation
/// 
/// This simulation explores how different Zipf distribution parameters affect
/// system performance. The Zipf distribution models access patterns
/// where some accounts are accessed much more frequently than others.
/// 
/// The sweep varies the Zipf parameter (α) from 0.0 (uniform distribution) to higher values,
/// running multiple simulations to understand how access pattern skewness affects
/// transaction throughput, contention, and overall system performance.
pub async fn run_sweep_zipf_simulation() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_zipf.toml
    let sweep_config = load_config()?;
    
    // Calculate Zipf parameters for each simulation using the helper function
    // Creates a sequence of Zipf parameters: 0.0, 0.1, 0.2, 0.3, etc.
    // Each value represents the skewness of the access distribution
    let zipf_parameters = generate_f64_sequence(
        0.0,  // Start at 0.0 (uniform distribution)
        sweep_config.sweep.zipf_step,
        sweep_config.sweep.num_simulations
    );

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Zipf Distribution",           // Human-readable name for logging
        "sim_sweep_zipf",              // Directory name for results
        "zipf_parameter",              // Parameter name for JSON output
        zipf_parameters,               // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, zipf_param| {
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: base_config.network_config.clone(),
                    account_config: base_config.account_config.clone(),
                    transaction_config: crate::config::TransactionConfig {
                        target_tps: base_config.transaction_config.target_tps,
                        sim_total_block_number: base_config.transaction_config.sim_total_block_number,
                        zipf_parameter: zipf_param,  // This is the parameter we're varying
                        ratio_cats: base_config.transaction_config.ratio_cats,
                        cat_lifetime_blocks: base_config.transaction_config.cat_lifetime_blocks,
                        initialization_wait_blocks: base_config.transaction_config.initialization_wait_blocks,
                        allow_cat_pending_dependencies: base_config.transaction_config.allow_cat_pending_dependencies,
                    },
                    repeat_config: base_config.repeat_config.clone(),
                }
            })
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "zipf_parameter", all_results)
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
/// This function provides the configuration needed to register the Zipf distribution sweep
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepZipf, SimulationConfig {
        name: "Zipf Distribution Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_zipf_simulation().await
                .map_err(|e| format!("Zipf sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_zipf/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Run with Plotting
// ------------------------------------------------------------------------------------------------

/// Runs the Zipf parameter sweep simulation with automatic plotting.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_zipf_simulation(),
        "Zipf Parameter Sweep",
        "simulator/src/scenarios/sim_sweep_zipf/plot_results.py"
    ).await
} 