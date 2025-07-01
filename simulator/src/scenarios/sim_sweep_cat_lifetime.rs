use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

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
    let sweep_config = crate::config::Config::load_sweep_cat_lifetime()?;
    
    // Calculate CAT lifetimes for each simulation
    // Creates a sequence of lifetimes: 1 block, 2 blocks, 3 blocks, etc.
    // Each value represents the number of blocks a CAT remains valid
    let cat_lifetimes: Vec<u64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| (i as u64 + 1) * sweep_config.sweep.cat_lifetime_step.unwrap())
        .collect();

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "CAT Lifetime",                // Human-readable name for logging
        "sim_sweep_cat_lifetime",      // Directory name for results
        "cat_lifetime",                // Parameter name for JSON output
        cat_lifetimes,                 // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep_cat_lifetime().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        // This takes the base config and applies the current CAT lifetime
        Box::new(|sweep_config, cat_lifetime| {
            let config = sweep_config.as_any().downcast_ref::<crate::config::SweepCatLifetimeConfig>().unwrap();
            crate::config::Config {
                network: config.network.clone(),
                num_accounts: config.num_accounts.clone(),
                transactions: crate::config::TransactionConfig {
                    target_tps: config.transactions.target_tps,
                    sim_total_block_number: config.transactions.sim_total_block_number,
                    zipf_parameter: config.transactions.zipf_parameter,
                    ratio_cats: config.transactions.ratio_cats,
                    cat_lifetime_blocks: cat_lifetime,  // This is the parameter we're varying
                    initialization_wait_blocks: config.transactions.initialization_wait_blocks,
                    allow_cat_pending_dependencies: config.transactions.allow_cat_pending_dependencies,
                },
            }
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "cat_lifetime", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 