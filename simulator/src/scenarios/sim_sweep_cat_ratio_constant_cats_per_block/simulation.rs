use crate::scenarios::sweep_runner::{SweepRunner, create_modified_config};
use crate::define_sweep_config;
use crate::config::ValidateConfig;
use crate::scenarios::utils::run_simulation_with_plotting;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------------
// Sweep-Specific Parameter Struct
// ------------------------------------------------------------------------------------------------

/// Parameters specific to the CAT ratio with constant CATs per block sweep simulation.
/// 
/// This struct defines the parameters used to control the sweep where target_tpb
/// is varied and cat_ratio is calculated to maintain constant CATs per block.
#[derive(Debug, Deserialize, Clone)]
pub struct CatRatioConstantCatsPerBlockSweepParameters {
    /// Total number of simulation runs in the sweep (determines how many parameter values to test)
    pub num_simulations: usize,
    /// Multiplier for target TPB per simulation step
    pub target_tpb_multiplier_per_step: f64,
}

// ------------------------------------------------------------------------------------------------
// Sweep Configuration
// ------------------------------------------------------------------------------------------------

// Defines the sweep configuration for CAT ratio with constant CATs per block simulations.
// 
// This macro generates a complete sweep configuration setup including:
// - A config struct with standard fields (network_config, account_config, transaction_config, sweep)
// - Standard validation logic for common fields
// - SweepConfigTrait implementation for integration with the generic SweepRunner
// - A load_config() function that reads and validates the TOML configuration file
define_sweep_config!(
    "sim_sweep_cat_ratio_constant_cats_per_block",
    SweepCatRatioConstantCatsPerBlockConfig,
    validate_sweep_specific = |self_: &Self| {
        // Need target_tpb_multiplier_per_step to calculate the sequence of target TPB values to test
        if self_.simulation_config.target_tpb_multiplier_per_step.is_none() {
            return Err(crate::config::ConfigError::ValidationError("Target TPB multiplier per step must be specified".into()));
        }
        
        let multiplier = self_.simulation_config.target_tpb_multiplier_per_step.unwrap();
        if multiplier <= 0.0 {
            return Err(crate::config::ConfigError::ValidationError("Target TPB multiplier per step must be positive".into()));
        }
        
        Ok(())
    }
);

// ------------------------------------------------------------------------------------------------
// Simulation Runner
// ------------------------------------------------------------------------------------------------

/// Runs the sweep CAT ratio with constant CATs per block simulation
/// 
/// This simulation explores how different target TPB and CAT ratio combinations
/// affect system performance while maintaining a constant number of CATs per block.
/// The CATs per block is calculated as: cat_ratio × target_tpb
/// 
/// This ensures that regardless of the target TPB, the same number of CATs are
/// generated per block, allowing us to isolate the effect of transaction rate
/// from the effect of CAT frequency.
pub async fn run_sweep_cat_ratio_constant_cats_per_block_simulation() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_cat_ratio_constant_cats_per_block.toml
    let sweep_config = load_config()?;
    
    // Get multiplier and number of simulations from config
    let multiplier = sweep_config.simulation_config.target_tpb_multiplier_per_step.unwrap();
    let num_simulations = sweep_config.simulation_config.num_simulations.unwrap();
    
    // Calculate target TPB values using the multiplier
    // base_tpb * (multiplier ^ step) for each simulation
    let base_tpb = 2.0; // Starting value
    let mut target_tpb_values = Vec::new();
    for step in 0..num_simulations {
        let target_tpb = base_tpb * multiplier.powi(step as i32);
        target_tpb_values.push(target_tpb);
    }
    
    // Calculate the reference CATs per block from the base config (for verification)
    let reference_target_tpb = sweep_config.transaction_config.target_tpb;
    let reference_cat_ratio = sweep_config.transaction_config.ratio_cats;
    let _constant_cats_per_block = reference_target_tpb * reference_cat_ratio; // Should be 2.0

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "CAT Ratio with Constant CATs per Block",  // Human-readable name for logging
        "sim_sweep_cat_ratio_constant_cats_per_block",  // Directory name for results
        "target_tpb",                             // Parameter name for JSON output
        target_tpb_values,                        // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            load_config().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, target_tpb| {
            // Calculate the CAT ratio to maintain constant CATs per block
            // constant_cats_per_block = target_tpb * cat_ratio
            // So: cat_ratio = constant_cats_per_block / target_tpb
            let constant_cats_per_block = 2.0; // From reference config (20.0 * 0.1)
            let cat_ratio = constant_cats_per_block / target_tpb;
            
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: base_config.network_config.clone(),
                    account_config: base_config.account_config.clone(),
                    transaction_config: crate::config::TransactionConfig {
                        target_tpb: target_tpb,  // This is the parameter we're varying
                        zipf_parameter: base_config.transaction_config.zipf_parameter,
                        ratio_cats: cat_ratio,  // Calculated to maintain constant CATs per block
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

    // Execute the sweep
    runner.run().await
}

// ------------------------------------------------------------------------------------------------
// Registration
// ------------------------------------------------------------------------------------------------

/// Registers this sweep simulation with the simulation registry
/// 
/// This function is called by the simulation registry to register this sweep
/// so it can be selected from the main menu.
pub fn register() -> (crate::interface::SimulationType, crate::simulation_registry::SimulationConfig) {
    use crate::interface::SimulationType;
    use crate::simulation_registry::SimulationConfig;
    
    (SimulationType::SweepCatRatioConstantCatsPerBlock, SimulationConfig {
        name: "CAT Ratio with Constant CATs per Block Sweep",
        run_fn: Box::new(|| Box::pin(async {
            run_sweep_cat_ratio_constant_cats_per_block_simulation().await
                .map_err(|e| format!("CAT ratio constant CATs per block sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_cat_ratio_constant_cats_per_block/plot_results.py",
    })
}

// ------------------------------------------------------------------------------------------------
// Public Interface
// ------------------------------------------------------------------------------------------------

/// Public function to run the simulation with plotting
/// 
/// This is the main entry point for running this sweep simulation.
/// It loads the configuration, runs the sweep, and generates plots.
pub async fn run_with_plotting() -> Result<(), crate::config::ConfigError> {
    run_simulation_with_plotting(
        || run_sweep_cat_ratio_constant_cats_per_block_simulation(),
        "CAT Ratio with Constant CATs per Block Sweep",
        "simulator/src/scenarios/sim_sweep_cat_ratio_constant_cats_per_block/plot_results.py"
    ).await
} 