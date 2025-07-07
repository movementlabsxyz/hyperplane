use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, SweepConfigTrait};

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

/// Implementation of SweepConfigTrait for CAT pending dependencies sweep configurations.
/// 
/// This allows the SweepRunner to work with configurations specifically designed
/// for CAT pending dependencies sweeps.
impl SweepConfigTrait for crate::config::SweepCatPendingDependenciesConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network_config(&self) -> &crate::config::NetworkConfig { &self.network_config }
    fn get_account_config(&self) -> &crate::config::AccountConfig { &self.account_config }
    fn get_transaction_config(&self) -> &crate::config::TransactionConfig { &self.transaction_config }
    fn as_any(&self) -> &dyn std::any::Any { self }
} 