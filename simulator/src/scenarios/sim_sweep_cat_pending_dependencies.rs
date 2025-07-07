use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, SweepConfigTrait};
use crate::config::{ValidateConfig, NetworkConfig, AccountConfig, TransactionConfig, SweepParameters, ConfigError};
use std::fs;
use toml;
use serde::Deserialize;
use std::any::Any;

// ============================================================================
// Sweep Configuration
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct SweepCatPendingDependenciesConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

impl ValidateConfig for SweepCatPendingDependenciesConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        crate::config::validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        // For this sweep, we only need to validate that num_simulations is set
        // The sweep will test exactly 2 values: false and true
        if self.sweep.num_simulations != 2 {
            return Err(ConfigError::ValidationError("Number of simulations must be exactly 2 for CAT pending dependencies sweep (false and true)".into()));
        }
        Ok(())
    }
}

impl SweepConfigTrait for SweepCatPendingDependenciesConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_num_simulations(&self) -> usize {
        self.sweep.num_simulations
    }
    fn get_network_config(&self) -> &NetworkConfig {
        &self.network_config
    }
    fn get_account_config(&self) -> &AccountConfig {
        &self.account_config
    }
    fn get_transaction_config(&self) -> &TransactionConfig {
        &self.transaction_config
    }
}

// ------------------------------------------------------------------------------------------------
// Configuration Loading
// ------------------------------------------------------------------------------------------------

/// Loads the CAT pending dependencies sweep configuration from the TOML file.
/// 
/// This function reads the configuration file and validates it according to
/// the sweep-specific validation rules.
fn load_config() -> Result<SweepCatPendingDependenciesConfig, crate::config::ConfigError> {
    let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_cat_pending_dependencies.toml")?;
    let config: SweepCatPendingDependenciesConfig = toml::from_str(&config_str)?;
    config.validate()?;
    Ok(config)
}

// ------------------------------------------------------------------------------------------------
// Parameter Sequence Generation & Sweep Runner Setup
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
        plot_script: "simulator/scripts/sim_sweep_cat_pending_dependencies/plot_results.py",
    })
} 