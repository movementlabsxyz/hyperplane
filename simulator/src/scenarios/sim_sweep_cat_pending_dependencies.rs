use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------------
// Sweep-Specific Parameter Struct
// ------------------------------------------------------------------------------------------------

/// Parameters specific to the CAT pending dependencies sweep simulation.
/// 
/// This struct defines the parameters used to control the CAT pending dependencies sweep.
/// It contains only the parameters relevant to this specific sweep type.
#[derive(Debug, Deserialize, Clone)]
pub struct CatPendingDependenciesSweepParameters {
    /// Total number of simulation runs in the sweep (must be exactly 2 for false/true values)
    pub num_simulations: usize,
}

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

define_sweep_config!(
    SweepCatPendingDependenciesConfig,
    "config_sweep_cat_pending_dependencies.toml",
    sweep_parameters = CatPendingDependenciesSweepParameters,
    validate_sweep_specific = |self_: &Self| {
        // Need exactly 2 simulations to test false and true values for the flag
        if self_.sweep.num_simulations != 2 {
            return Err(crate::config::ConfigError::ValidationError("Number of simulations must be exactly 2 for CAT pending dependencies sweep (false and true)".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

/// Runs the sweep CAT pending dependencies simulation
/// 
/// This simulation explores how the ALLOW_CAT_PENDING_DEPENDENCIES flag affects
/// system performance. The flag controls whether CAT transactions can depend
/// on locked keys.
/// 
/// The sweep tests exactly two values:
/// - false: CATs are rejected when they depend on locked keys
/// - true: CATs are allowed to depend on locked keys (current behavior)
/// 
/// This helps understand the impact of this restriction on transaction throughput,
/// contention, and overall system performance.
pub async fn run_sweep_cat_pending_dependencies_simulation() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_cat_pending_dependencies.toml
    let _sweep_config = load_config()?;
    
    // Create the two values to test: false and true
    let allow_cat_pending_dependencies_values: Vec<bool> = vec![false, true];

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "CAT Pending Dependencies",    // Human-readable name for logging
        "sim_sweep_cat_pending_dependencies", // Directory name for results
        "allow_cat_pending_dependencies", // Parameter name for JSON output
        allow_cat_pending_dependencies_values, // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, allow_cat_pending_dependencies| {
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: base_config.network_config.clone(),
                    account_config: base_config.account_config.clone(),
                    transaction_config: crate::config::TransactionConfig {
                        target_tps: base_config.transaction_config.target_tps,
                        sim_total_block_number: base_config.transaction_config.sim_total_block_number,
                        zipf_parameter: base_config.transaction_config.zipf_parameter,
                        ratio_cats: base_config.transaction_config.ratio_cats,
                        cat_lifetime_blocks: base_config.transaction_config.cat_lifetime_blocks,
                        initialization_wait_blocks: base_config.transaction_config.initialization_wait_blocks,
                        allow_cat_pending_dependencies: allow_cat_pending_dependencies,  // This is the parameter we're varying
                    },
                }
            })
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "allow_cat_pending_dependencies", all_results)
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
/// This function provides the configuration needed to register the CAT pending dependencies sweep
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepCatPendingDependencies, SimulationConfig {
        name: "CAT Pending Dependencies Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_cat_pending_dependencies_simulation().await
                .map_err(|e| format!("CAT pending dependencies sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_cat_pending_dependencies/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Run with Plotting
// ------------------------------------------------------------------------------------------------

/// Runs the CAT pending dependencies sweep simulation with automatic plotting.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_cat_pending_dependencies_simulation(),
        "CAT Pending Dependencies Sweep",
        "simulator/src/scenarios/sim_sweep_cat_pending_dependencies/plot_results.py"
    ).await
} 