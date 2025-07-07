use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_f64_sequence};
use crate::config::ValidateConfig;
use std::fs;
use toml;

/// Loads the Zipf sweep configuration from the TOML file.
/// 
/// This function reads the configuration file and validates it according to
/// the sweep-specific validation rules.
fn load_config() -> Result<crate::config::SweepZipfConfig, crate::config::ConfigError> {
    let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_zipf.toml")?;
    let config: crate::config::SweepZipfConfig = toml::from_str(&config_str)?;
    config.validate()?;
    Ok(config)
}

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
        sweep_config.sweep.zipf_step.unwrap(),
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
        plot_script: "simulator/scripts/sim_sweep_zipf/plot_results.py",
    })
} 