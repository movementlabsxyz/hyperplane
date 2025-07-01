use crate::scenarios::sweep_runner::{SweepRunner, save_generic_sweep_results};

/// Runs the sweep duration simulation
/// 
/// This simulation explores how different simulation durations affect the simulation. 
/// 
/// The sweep varies the simulation duration from a minimum to longer periods.
pub async fn run_sweep_duration() -> Result<(), crate::config::ConfigError> {
    // Load sweep configuration to get parameter values
    // This reads the sweep settings from config_sweep_duration.toml
    let sweep_config = crate::config::Config::load_sweep_duration()?;
    
    // Calculate durations for each simulation
    // Creates a sequence of durations: 5s, 10s, 15s, 20s, etc.
    // Each value represents the total simulation time in seconds
    let durations: Vec<u64> = (0..sweep_config.sweep.num_simulations)
        .map(|i| 5 + (i as u64 * sweep_config.sweep.duration_step.unwrap()))
        .collect();

    // Create the generic sweep runner that handles all the common functionality
    // This eliminates code duplication across different sweep types
    let runner = SweepRunner::new(
        "Duration",                    // Human-readable name for logging
        "sim_sweep_duration",          // Directory name for results
        "duration",                    // Parameter name for JSON output
        durations,                     // List of parameter values to test
        // Function to load the sweep configuration
        Box::new(|| {
            crate::config::Config::load_sweep_duration().map(|config| Box::new(config) as Box<dyn crate::scenarios::sweep_runner::SweepConfigTrait>)
        }),
        // Function to create a modified config for each simulation
        // This takes the base config and applies the current duration
        Box::new(|sweep_config, duration| {
            let config = sweep_config.as_any().downcast_ref::<crate::config::SweepDurationConfig>().unwrap();
            crate::config::Config {
                network: config.network.clone(),
                num_accounts: config.num_accounts.clone(),
                transactions: crate::config::TransactionConfig {
                    target_tps: config.transactions.target_tps,
                    duration_seconds: duration,  // This is the parameter we're varying
                    zipf_parameter: config.transactions.zipf_parameter,
                    ratio_cats: config.transactions.ratio_cats,
                    cat_lifetime_blocks: config.transactions.cat_lifetime_blocks,
                    initialization_wait_blocks: config.transactions.initialization_wait_blocks,
                },
            }
        }),
        // Function to save the combined results from all simulations
        Box::new(|results_dir, all_results| {
            save_generic_sweep_results(results_dir, "duration", all_results)
        }),
    );

    // Run the sweep - this handles all the simulation execution, logging, and result saving
    runner.run().await
} 