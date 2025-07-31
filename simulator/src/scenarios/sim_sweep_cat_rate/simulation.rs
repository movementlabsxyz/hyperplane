use crate::scenarios::sweep_runner::{SweepRunner, create_modified_config, generate_f64_sequence};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------------
// Sweep-Specific Parameter Struct
// ------------------------------------------------------------------------------------------------

/// Parameters specific to the CAT rate sweep simulation.
/// 
/// This struct defines the parameters used to control the CAT rate sweep.
/// It contains only the parameters relevant to this specific sweep type.
#[derive(Debug, Deserialize, Clone)]
pub struct CatRateSweepParameters {
    /// Total number of simulation runs in the sweep (determines how many parameter values to test)
    pub num_simulations: usize,
    /// Step size for CAT ratio sweeps (0.0 = no CATs, 1.0 = all CATs)
    pub cat_rate_step: f64,
}

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

// Defines the sweep configuration for CAT rate simulations.
// 
// This macro generates a complete sweep configuration setup including:
// - A config struct with standard fields (network_config, account_config, transaction_config, sweep)
// - Standard validation logic for common fields
// - SweepConfigTrait implementation for integration with the generic SweepRunner
// - A load_config() function that reads and validates the TOML configuration file
define_sweep_config!(
    "sim_sweep_cat_rate",
    SweepCatRateConfig,
    validate_sweep_specific = |self_: &Self| {
        // Need cat_rate_step to generate the sequence of CAT ratios to test
        if self_.simulation_config.cat_rate_step.unwrap_or(0.0) <= 0.0 {
            return Err(crate::config::ConfigError::ValidationError("CAT rate step must be positive".into()));
        }
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

/// Runs the sweep CAT rate simulation
/// 
/// This simulation explores how different CAT (Cross-Chain Atomic Transaction) ratios affect
/// system performance. CATs are transactions that must succeed on all chains or fail on all chains.
/// 
/// The sweep varies the ratio of CAT transactions from 0.0 (no CATs) to a maximum value,
/// running multiple simulations to understand the impact on transaction throughput,
/// success rates, and system behavior.
pub async fn run_sweep_cat_rate_simulation() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_cat_rate.toml
    let sweep_config = load_config()?;
    
    // Calculate CAT ratios for each simulation using the helper function
    // Creates a sequence of CAT ratios starting from the base ratio_cats value
    // Each value represents the fraction of transactions that should be CATs
    let cat_ratios = generate_f64_sequence(
        sweep_config.transaction_config.ratio_cats, 
        sweep_config.simulation_config.cat_rate_step.unwrap(),
        sweep_config.simulation_config.num_simulations.unwrap()
    );

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "CAT Rate",                    // Human-readable name for logging
        "sim_sweep_cat_rate",          // Directory name for results
        "cat_ratio",                   // Parameter name for JSON output
        cat_ratios,                    // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, cat_ratio| {
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: base_config.network_config.clone(),
                    account_config: base_config.account_config.clone(),
                    transaction_config: crate::config::TransactionConfig {
                        target_tpb: base_config.transaction_config.target_tpb,
                        zipf_parameter: base_config.transaction_config.zipf_parameter,
                        ratio_cats: cat_ratio,  // This is the parameter we're varying
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
/// This function provides the configuration needed to register the CAT rate sweep
/// with the main simulation registry.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepCatRate, SimulationConfig {
        name: "CAT Rate Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_cat_rate_simulation().await
                .map_err(|e| format!("CAT rate sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_cat_rate/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Run with Plotting
// ------------------------------------------------------------------------------------------------

/// Runs the CAT rate sweep simulation with automatic plotting.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_cat_rate_simulation(),
        "CAT Rate Sweep",
        "simulator/src/scenarios/sim_sweep_cat_rate/plot_results.py"
    ).await
} 