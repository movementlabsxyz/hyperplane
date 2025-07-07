use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results, create_modified_config, generate_u64_sequence};


/// Runs the sweep total block number simulation
/// 
/// This simulation explores how different simulation block counts affect the simulation. 
/// 
/// The sweep varies the total number of blocks to simulate from a minimum to longer periods.
pub async fn run_sweep_total_block_number() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_total_block_number.toml
    let sweep_config = crate::config::Config::load_sweep_total_block_number()?;
    
    // Calculate block numbers for each simulation using the helper function
    // Creates a sequence of block numbers: 25, 50, 75, 100, etc.
    // Each value represents the total number of blocks to simulate
    let block_numbers = generate_u64_sequence(
        25,  // Start at 25 blocks
        sweep_config.sweep.duration_step.unwrap(),
        sweep_config.sweep.num_simulations
    );

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Duration",                    // Human-readable name for logging
        "sim_sweep_total_block_number",          // Directory name for results
        "duration",                    // Parameter name for JSON output
        block_numbers,                 // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep_total_block_number().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation using the helper
        Box::new(|sweep_config, block_number| {
            create_modified_config(sweep_config, |base_config| {
                crate::config::Config {
                    network_config: base_config.network_config.clone(),
                    account_config: base_config.account_config.clone(),
                    transaction_config: crate::config::TransactionConfig {
                        target_tps: base_config.transaction_config.target_tps,
                        sim_total_block_number: block_number,  // This is the parameter we're varying
                        zipf_parameter: base_config.transaction_config.zipf_parameter,
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
            save_generic_sweep_results(results_dir, "duration", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 