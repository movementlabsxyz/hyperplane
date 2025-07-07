use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_u64_sequence, SweepConfigTrait};
use crate::config::{ValidateConfig, NetworkConfig, AccountConfig, TransactionConfig, SweepParameters, ConfigError};
use std::fs;
use toml;
use serde::Deserialize;
use std::any::Any;

// ============================================================================
// Sweep Configuration
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct SweepCatLifetimeConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

impl ValidateConfig for SweepCatLifetimeConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        crate::config::validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        if self.sweep.cat_lifetime_step.is_none() {
            return Err(ConfigError::ValidationError("CAT lifetime step must be specified".into()));
        }
        Ok(())
    }
}

impl SweepConfigTrait for SweepCatLifetimeConfig {
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

/// Loads the CAT lifetime sweep configuration from the TOML file.
/// 
/// This function reads the configuration file and validates it according to
/// the sweep-specific validation rules.
fn load_config() -> Result<SweepCatLifetimeConfig, crate::config::ConfigError> {
    let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_cat_lifetime.toml")?;
    let config: SweepCatLifetimeConfig = toml::from_str(&config_str)?;
    config.validate()?;
    Ok(config)
}

// ------------------------------------------------------------------------------------------------
// Parameter Sequence Generation & Sweep Runner Setup
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
    // Creates a sequence of lifetimes: 1 block, 2 blocks, 3 blocks, etc.
    // Each value represents the number of blocks a CAT remains valid
    let cat_lifetimes = generate_u64_sequence(
        sweep_config.sweep.cat_lifetime_step.unwrap(),
        sweep_config.sweep.cat_lifetime_step.unwrap(),
        sweep_config.sweep.num_simulations
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
                        sim_total_block_number: base_config.transaction_config.sim_total_block_number,
                        zipf_parameter: base_config.transaction_config.zipf_parameter,
                        ratio_cats: base_config.transaction_config.ratio_cats,
                        cat_lifetime_blocks: cat_lifetime,  // This is the parameter we're varying
                        initialization_wait_blocks: base_config.transaction_config.initialization_wait_blocks,
                        allow_cat_pending_dependencies: base_config.transaction_config.allow_cat_pending_dependencies,
                    },
                }
            })
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "cat_lifetime", all_results)
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
        plot_script: "simulator/scripts/sim_sweep_cat_lifetime/plot_results.py",
    })
} 