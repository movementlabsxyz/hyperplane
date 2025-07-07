use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

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
    let _sweep_config = crate::config::Config::load_sweep_cat_pending_dependencies()?;
    
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
            crate::config::Config::load_sweep_cat_pending_dependencies().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        // This takes the base config and applies the current allow_cat_pending_dependencies value
        Box::new(|sweep_config, allow_cat_pending_dependencies| {
            let config = sweep_config.as_any().downcast_ref::<crate::config::SweepCatPendingDependenciesConfig>().unwrap();
            crate::config::Config {
                network_config: config.network.clone(),
                account_config: config.num_accounts.clone(),
                transaction_config: crate::config::TransactionConfig {
                    target_tps: config.transactions.target_tps,
                    sim_total_block_number: config.transactions.sim_total_block_number,
                    zipf_parameter: config.transactions.zipf_parameter,
                    ratio_cats: config.transactions.ratio_cats,
                    cat_lifetime_blocks: config.transactions.cat_lifetime_blocks,
                    initialization_wait_blocks: config.transactions.initialization_wait_blocks,
                    allow_cat_pending_dependencies: allow_cat_pending_dependencies,  // This is the parameter we're varying
                },
            }
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "allow_cat_pending_dependencies", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 