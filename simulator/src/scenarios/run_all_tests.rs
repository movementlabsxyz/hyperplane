use std::time::Instant;
use hyperplane::utils::logging;

/// Runs all simulation tests sequentially and generates plots after each
pub async fn run_all_tests() -> Result<(), crate::config::ConfigError> {
    let start_time = Instant::now();
    
    logging::log("SIMULATOR", "=== Starting All Tests Suite ===");
    logging::log("SIMULATOR", "This will run all simulation types sequentially");
    
    // Run simple simulation
    crate::scenarios::sim_simple::run_with_plotting().await?;
    
    // Run CAT rate sweep
    crate::scenarios::sim_sweep_cat_rate::run_with_plotting().await?;
    
    // Run Zipf sweep
    crate::scenarios::sim_sweep_zipf::run_with_plotting().await?;
    
    // Run chain delay sweep
    crate::scenarios::sim_sweep_chain_delay::run_with_plotting().await?;
    
    // Run duration sweep
    crate::scenarios::sim_sweep_total_block_number::run_with_plotting().await?;
    
    // Run CAT lifetime sweep
    crate::scenarios::sim_sweep_cat_lifetime::run_with_plotting().await?;
    
    // Run block interval constant block delay sweep
    crate::scenarios::sim_sweep_block_interval_constant_block_delay::run_with_plotting().await?;
    
    // Run block interval constant time delay sweep
    crate::scenarios::sim_sweep_block_interval_constant_time_delay::run_with_plotting().await?;
    
    // Run CAT pending dependencies sweep
    crate::scenarios::sim_sweep_cat_pending_dependencies::run_with_plotting().await?;
    
    let total_time = start_time.elapsed();
    logging::log("SIMULATOR", "=== All Tests Completed Successfully ===");
    logging::log("SIMULATOR", &format!("Total execution time: {:.2?}", total_time));
    
    println!("All tests completed successfully!");
    println!("Total execution time: {:.2?}", total_time);
    
    Ok(())
} 