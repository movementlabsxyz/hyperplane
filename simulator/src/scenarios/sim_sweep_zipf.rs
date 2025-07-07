use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_f64_sequence, SweepConfigTrait};

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
            crate::config::Config::load_sweep_zipf().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
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

/// Implementation of SweepConfigTrait for Zipf distribution sweep configurations.
/// 
/// This allows the SweepRunner to work with configurations specifically designed
/// for Zipf parameter sweeps.
impl SweepConfigTrait for crate::config::SweepZipfConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network_config(&self) -> &crate::config::NetworkConfig { &self.network_config }
    fn get_account_config(&self) -> &crate::config::AccountConfig { &self.account_config }
    fn get_transaction_config(&self) -> &crate::config::TransactionConfig { &self.transaction_config }
    fn as_any(&self) -> &dyn std::any::Any { self }
} 