use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

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
    let sweep_config = crate::config::Config::load_sweep()?;
    
    // Calculate CAT ratios for each simulation
    // Creates a sequence of CAT ratios: 0.0, 0.1, 0.2, 0.3, etc.
    // Each value represents the fraction of transactions that should be CATs
    let cat_ratios: Vec<f64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| i as f64 * sweep_config.sweep.cat_rate_step.unwrap())
        .collect();

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "CAT Rate",                    // Human-readable name for logging
        "sim_sweep_cat_rate",          // Directory name for results
        "cat_ratio",                   // Parameter name for JSON output
        cat_ratios,                    // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        // This takes the base config and applies the current CAT ratio
        Box::new(|sweep_config, cat_ratio| {
            let config = sweep_config.as_any().downcast_ref::<crate::config::SweepConfig>().unwrap();
            crate::config::Config {
                network: config.network.clone(),
                num_accounts: config.num_accounts.clone(),
                transactions: crate::config::TransactionConfig {
                    target_tps: config.transactions.target_tps,
                    sim_total_block_number: config.transactions.sim_total_block_number,
                    zipf_parameter: config.transactions.zipf_parameter,
                    ratio_cats: cat_ratio,  // This is the parameter we're varying
                    cat_lifetime_blocks: config.transactions.cat_lifetime_blocks,
                    initialization_wait_blocks: config.transactions.initialization_wait_blocks,
                },
            }
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "cat_ratio", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 