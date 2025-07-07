use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

/// Creates a configuration for Zipf distribution sweep.
/// 
/// This function takes a sweep configuration and a Zipf parameter value, then creates
/// a new Config with the Zipf parameter applied to the transaction configuration.
/// 
/// # Arguments
/// 
/// * `sweep_config` - The sweep configuration containing base parameters
/// * `zipf_param` - The Zipf parameter value to apply (controls access pattern skewness)
/// 
/// # Returns
/// 
/// A new Config with the Zipf parameter applied
fn create_zipf_config(
    sweep_config: &Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>,
    zipf_param: f64,
) -> crate::config::Config {
    let config = sweep_config.as_any().downcast_ref::<crate::config::SweepZipfConfig>().unwrap();
    crate::config::Config {
        network_config: config.network_config.clone(),
        account_config: config.account_config.clone(),
        transaction_config: crate::config::TransactionConfig {
            target_tps: config.transaction_config.target_tps,
            sim_total_block_number: config.transaction_config.sim_total_block_number,
            zipf_parameter: zipf_param,  // This is the parameter we're varying
            ratio_cats: config.transaction_config.ratio_cats,
            cat_lifetime_blocks: config.transaction_config.cat_lifetime_blocks,
            initialization_wait_blocks: config.transaction_config.initialization_wait_blocks,
            allow_cat_pending_dependencies: config.transaction_config.allow_cat_pending_dependencies,
        },
    }
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
    let sweep_config = crate::config::Config::load_sweep_zipf()?;
    
    // Calculate Zipf parameters for each simulation
    // Creates a sequence of Zipf parameters: 0.0, 0.1, 0.2, 0.3, etc.
    // Each value represents the skewness of the access distribution
    let zipf_parameters: Vec<f64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| i as f64 * sweep_config.sweep.zipf_step.unwrap())
        .collect();

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Zipf Distribution",           // Human-readable name for logging
        "sim_sweep_zipf",              // Directory name for results
        "zipf_parameter",              // Parameter name for JSON output
        zipf_parameters,               // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep_zipf().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        Box::new(|sweep_config, zipf_param| {
            create_zipf_config(sweep_config, zipf_param)
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "zipf_parameter", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 