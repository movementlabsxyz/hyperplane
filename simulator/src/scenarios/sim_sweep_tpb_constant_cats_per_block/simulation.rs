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
    "sim_sweep_tpb_constant_cats_per_block",
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
pub async fn run_sweep_tpb_constant_cats_per_block_simulation() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_cat_ratio_constant_cats_per_block.toml
    let sweep_config = load_config()?;
    
    // Get multiplier and number of simulations from config
    let multiplier = sweep_config.simulation_config.target_tpb_multiplier_per_step.unwrap();
    let num_simulations = sweep_config.simulation_config.num_simulations.unwrap();
    
    // Calculate target TPB values using the multiplier
    // base_tpb * (multiplier ^ step) for each simulation
    let base_tpb = 10.0; // Starting value (will give block_interval = 0.1)
    let mut target_tpb_values = Vec::new();
    for step in 0..num_simulations {
        let target_tpb = base_tpb * multiplier.powi(step as i32);
        target_tpb_values.push(target_tpb);
    }
    
    // Get the constant CATs per block from the simulation config
    let constant_cats_per_block = sweep_config.simulation_config.constants_cats_per_block.unwrap_or(10.0);
    
    // Calculate CAT TPB values (constant)
    let cat_tpb_values: Vec<f64> = vec![constant_cats_per_block; num_simulations];
    
    // Calculate corresponding block interval values
    let block_interval_values: Vec<f64> = target_tpb_values.iter().map(|&tpb| tpb / 100.0).collect();
    
    // Print target TPB, block interval, and CAT TPB values
    println!("Target TPB values to test: {:?}", target_tpb_values);
    println!("Block interval values (scaled): {:?}", block_interval_values);
    println!("CAT TPB values (constant): {:?}", cat_tpb_values);

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "CAT Ratio with Constant CATs per Block",  // Human-readable name for logging
        "sim_sweep_tpb_constant_cats_per_block",  // Directory name for results
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
            let cat_ratio = 10.0 / target_tpb; // Using hardcoded value for now
            
            // Scale block_interval with target_tpb
            // We want: target_tpb=10 -> block_interval=0.1, target_tpb=100 -> block_interval=1.0
            // So: block_interval = target_tpb / 100
            let scaled_block_interval = target_tpb / 100.0;
            
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: crate::config::NetworkConfig {
                        num_chains: base_config.network_config.num_chains,
                        chain_delays: base_config.network_config.chain_delays.clone(),
                        block_interval: scaled_block_interval,  // Scaled with target_tpb
                        channel_buffer_size: base_config.network_config.channel_buffer_size,
                    },
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
            run_sweep_tpb_constant_cats_per_block_simulation().await
                .map_err(|e| format!("CAT ratio constant CATs per block sweep failed: {}", e))
        })),
        plot_script: "simulator/src/scenarios/sim_sweep_tpb_constant_cats_per_block/plot_results.py",
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
        || run_sweep_tpb_constant_cats_per_block_simulation(),
        "CAT Ratio with Constant CATs per Block Sweep",
        "simulator/src/scenarios/sim_sweep_tpb_constant_cats_per_block/plot_results.py"
    ).await
} 