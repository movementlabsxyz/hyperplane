use std::time::Instant;
use hyperplane::utils::logging;

/// Runs all simulation tests sequentially and generates plots after each
pub async fn run_all_tests() -> Result<(), crate::config::ConfigError> {
    let start_time = Instant::now();
    
    logging::log("SIMULATOR", "=== Starting All Tests Suite ===");
    logging::log("SIMULATOR", "This will run all simulation types sequentially");
    
    // 1. Simple simulation
    println!("\n------------ 1. Simple Simulation -----------");
    logging::log("SIMULATOR", "------------ 1. Simple Simulation -----------");
    crate::scenarios::sim_simple::simulation::run_with_plotting().await?;
    
    // 2. Simple repeated simulation
    println!("\n------------ 2. Simple Simulation -----------");
    logging::log("SIMULATOR", "------------ 2. Simple Simulation -----------");
    crate::scenarios::sim_simple::simulation::run_with_plotting().await?;
    
    // 3. CAT rate sweep
    println!("\n------------ 3. Sweep CAT Rate -----------");
    logging::log("SIMULATOR", "------------ 3. Sweep CAT Rate -----------");
    crate::scenarios::sim_sweep_cat_rate::simulation::run_with_plotting().await?;
    
    // 4. Zipf sweep
    println!("\n------------ 4. Sweep Zipf Distribution -----------");
    logging::log("SIMULATOR", "------------ 4. Sweep Zipf Distribution -----------");
    crate::scenarios::sim_sweep_zipf::simulation::run_with_plotting().await?;
    
    // 5. Chain delay sweep
    println!("\n------------ 5. Sweep Chain Delay -----------");
    logging::log("SIMULATOR", "------------ 5. Sweep Chain Delay -----------");
    crate::scenarios::sim_sweep_chain_delay::simulation::run_with_plotting().await?;
    
    // 6. Total block number sweep
    println!("\n------------ 6. Sweep Total Block Number -----------");
    logging::log("SIMULATOR", "------------ 6. Sweep Total Block Number -----------");
    crate::scenarios::sim_sweep_total_block_number::simulation::run_with_plotting().await?;
    
    // 7. CAT lifetime sweep
    println!("\n------------ 7. Sweep CAT Lifetime -----------");
    logging::log("SIMULATOR", "------------ 7. Sweep CAT Lifetime -----------");
    crate::scenarios::sim_sweep_cat_lifetime::simulation::run_with_plotting().await?;
    
    // 8. Block interval constant block delay sweep
    println!("\n------------ 8. Sweep Block Interval (Constant Block Delay) -----------");
    logging::log("SIMULATOR", "------------ 8. Sweep Block Interval (Constant Block Delay) -----------");
    crate::scenarios::sim_sweep_block_interval_constant_block_delay::simulation::run_with_plotting().await?;
    
    // 9. Block interval constant time delay sweep
    println!("\n------------ 9. Sweep Block Interval (Constant Time Delay) -----------");
    logging::log("SIMULATOR", "------------ 9. Sweep Block Interval (Constant Time Delay) -----------");
    crate::scenarios::sim_sweep_block_interval_constant_time_delay::simulation::run_with_plotting().await?;
    
    // 10. CAT pending dependencies sweep
    println!("\n------------ 10. Sweep CAT Pending Dependencies -----------");
    logging::log("SIMULATOR", "------------ 10. Sweep CAT Pending Dependencies -----------");
    crate::scenarios::sim_sweep_cat_pending_dependencies::simulation::run_with_plotting().await?;
    
    let total_time = start_time.elapsed();
    logging::log("SIMULATOR", "=== All Tests Completed Successfully ===");
    logging::log("SIMULATOR", &format!("Total execution time: {:.2?}", total_time));
    
    println!("All tests completed successfully!");
    println!("Total execution time: {:.2?}", total_time);
    
    Ok(())
} 