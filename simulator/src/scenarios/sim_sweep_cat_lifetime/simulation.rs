use crate::scenarios::sweep_runner::{SweepRunner, create_modified_config, generate_u64_sequence};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

// Defines the sweep configuration for CAT lifetime simulations.
// 
// This macro generates a complete sweep configuration setup including:
// - A config struct with standard fields (network_config, account_config, transaction_config, simulation_config)
// - Standard validation logic for common fields
// - SweepConfigTrait implementation for integration with the generic SweepRunner
// - A load_config() function that reads and validates the TOML configuration file
define_sweep_config!(
    "sim_sweep_cat_lifetime",
    SweepCatLifetimeConfig,
    validate_sweep_specific = |self_: &Self| {
        // Need cat_lifetime_step to generate the sequence of CAT lifetimes to test
        if self_.simulation_config.cat_lifetime_step.unwrap_or(0) == 0 {
            return Err(crate::config::ConfigError::ValidationError("CAT lifetime step must be positive".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

/// Runs the sweep CAT lifetime simulation
/// 
/// This simulation explores how different CAT (Cross-Chain Atomic Transaction) lifetimes
/// affect system performance. CAT lifetime determines how long a cross-chain transaction
/// remains valid before it expires.
/// 
/// The sweep varies the CAT lifetime from a minimum number of blocks to longer periods,
/// running multiple simulations to understand how transaction expiration affects
/// success rates, retry patterns, and overall system throughput.
pub async fn run_sweep_cat_lifetime_simulation() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_cat_lifetime.toml
    let sweep_config = load_config()?;
    
    // Calculate CAT lifetimes for each simulation using the helper function
    // Creates a sequence starting from cat_lifetime_blocks and stepping by cat_lifetime_step
    // Each value represents the number of blocks a CAT remains valid
    let cat_lifetimes = generate_u64_sequence(
        sweep_config.transaction_config.cat_lifetime_blocks,  // Start from the configured cat_lifetime_blocks
        sweep_config.simulation_config.cat_lifetime_step.unwrap(),
        sweep_config.simulation_config.num_simulations.unwrap_or(1)
    );

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "CAT Lifetime",                // Human-readable name for logging
        "sim_sweep_cat_lifetime",      // Directory name for results
        "cat_lifetime",                // Parameter name for JSON output
        cat_lifetimes,                 // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, cat_lifetime| {
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: base_config.network_config.clone(),
                    account_config: base_config.account_config.clone(),
                    transaction_config: crate::config::TransactionConfig {
                        target_tps: base_config.transaction_config.target_tps,
                        zipf_parameter: base_config.transaction_config.zipf_parameter,
                        ratio_cats: base_config.transaction_config.ratio_cats,
                        cat_lifetime_blocks: cat_lifetime,  // This is the parameter we're varying
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
/// This function provides the configuration needed to register the CAT lifetime sweep
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepCatLifetime, SimulationConfig {
        name: "CAT Lifetime Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_cat_lifetime_simulation().await
                .map_err(|e| format!("CAT lifetime sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_cat_lifetime/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Run with Plotting
// ------------------------------------------------------------------------------------------------

/// Runs the CAT lifetime sweep simulation with automatic plotting.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_cat_lifetime_simulation(),
        "CAT Lifetime Sweep",
        "simulator/src/scenarios/sim_sweep_cat_lifetime/plot_results.py"
    ).await
} 