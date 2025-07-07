// Runs the sweep CAT rate simulation
// 
// This simulation explores how different CAT (Cross-Chain Atomic Transaction) ratios affect
// system performance. CATs are transactions that must succeed on all chains or fail on all chains.
// 
// The sweep varies the ratio of CAT transactions from 0.0 (no CATs) to a maximum value,
// running multiple simulations to understand the impact on transaction throughput,
// success rates, and system behavior.
sweep_simulation!(
    run_sweep_cat_rate_simulation,              // Function name for the generated async function
    "CAT Rate",                                 // Human-readable name for logging and progress messages
    "sim_sweep_cat_rate",                       // Directory name where results will be saved
    "cat_ratio",                                // Parameter name used in JSON output files
    crate::config::Config::load_sweep,          // Function that loads the sweep configuration from TOML
    crate::config::SweepConfig,                 // Type of the sweep configuration struct
    f64,                                        // Data type of the parameter being swept (CAT ratio as float)
    cat_rate_step,                              // Name of the step field in the sweep config (defines increment)
    |sweep_config, cat_ratio| {                // Closure that modifies config with current parameter value
        let config = sweep_config.as_any().downcast_ref::<crate::config::SweepConfig>().unwrap();
        crate::config::Config {
            network: config.network.clone(),
            num_accounts: config.num_accounts.clone(),
            transactions: crate::config::TransactionConfig {
                target_tps: config.transactions.target_tps,
                sim_total_block_number: config.transactions.sim_total_block_number,
                zipf_parameter: config.transactions.zipf_parameter,
                ratio_cats: cat_ratio,          // value that is swept over
                cat_lifetime_blocks: config.transactions.cat_lifetime_blocks,
                initialization_wait_blocks: config.transactions.initialization_wait_blocks,
                allow_cat_pending_dependencies: config.transactions.allow_cat_pending_dependencies,
            },
        }
    }
); 