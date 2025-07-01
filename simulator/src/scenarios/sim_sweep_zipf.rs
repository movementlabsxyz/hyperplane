use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

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
        // This takes the base config and applies the current Zipf parameter
        Box::new(|sweep_config, zipf_param| {
            let config = sweep_config.as_any().downcast_ref::<crate::config::SweepZipfConfig>().unwrap();
            crate::config::Config {
                network: config.network.clone(),
                num_accounts: config.num_accounts.clone(),
                transactions: crate::config::TransactionConfig {
                    target_tps: config.transactions.target_tps,
                    sim_total_block_number: config.transactions.sim_total_block_number,
                    zipf_parameter: zipf_param,  // This is the parameter we're varying
                    ratio_cats: config.transactions.ratio_cats,
                    cat_lifetime_blocks: config.transactions.cat_lifetime_blocks,
                    initialization_wait_blocks: config.transactions.initialization_wait_blocks,
                    allow_cat_pending_dependencies: config.transactions.allow_cat_pending_dependencies,
                },
            }
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "zipf_parameter", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 