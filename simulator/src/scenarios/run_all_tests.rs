use std::time::Instant;
use std::process::Command;
use hyperplane::utils::logging;

fn run_plot(script_path: &str, label: &str) {
    logging::log("PLOT", &format!("Generating plots for {}...", label));
    let output = Command::new("python3")
        .arg(script_path)
        .output();
    match output {
        Ok(output) if output.status.success() => {
            logging::log("PLOT", &format!("Plots for {} generated successfully!", label));
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr);
            logging::log("PLOT", &format!("Plot generation failed for {}: {}", label, err));
        }
        Err(e) => {
            logging::log("PLOT", &format!("Failed to execute plotting script for {}: {}", label, e));
        }
    }
}

/// Runs all simulation tests sequentially and generates plots after each
pub async fn run_all_tests() -> Result<(), crate::config::ConfigError> {
    let start_time = Instant::now();
    
    logging::log("SIMULATOR", "=== Starting All Tests Suite ===");
    logging::log("SIMULATOR", "This will run all simulation types sequentially");
    
    // Run simple simulation
    logging::log("SIMULATOR", "=== Running Simple Simulation ===");
    crate::run_simple_simulation().await.map_err(|e| {
        let error_context = format!("Simple Simulation failed: {}", e);
        crate::config::ConfigError::ValidationError(error_context)
    })?;
    logging::log("SIMULATOR", "Simple simulation completed successfully");
    run_plot("simulator/scripts/sim_simple/plot_results.py", "Simple Simulation");
    
    // Run CAT rate sweep
    logging::log("SIMULATOR", "=== Running CAT Rate Sweep ===");
    crate::run_sweep_cat_rate_simulation().await.map_err(|e| {
        let error_context = format!("CAT Rate Sweep failed: {}", e);
        crate::config::ConfigError::ValidationError(error_context)
    })?;
    logging::log("SIMULATOR", "CAT rate sweep completed successfully");
    run_plot("simulator/scripts/sim_sweep_cat_rate/plot_results.py", "CAT Rate Sweep");
    
    // Run Zipf sweep
    logging::log("SIMULATOR", "=== Running Zipf Parameter Sweep ===");
    crate::run_sweep_zipf_simulation().await.map_err(|e| {
        let error_context = format!("Zipf Parameter Sweep failed: {}", e);
        crate::config::ConfigError::ValidationError(error_context)
    })?;
    logging::log("SIMULATOR", "Zipf parameter sweep completed successfully");
    run_plot("simulator/scripts/sim_sweep_zipf/plot_results.py", "Zipf Parameter Sweep");
    
    // Run chain delay sweep
    logging::log("SIMULATOR", "=== Running Chain Delay Sweep ===");
    crate::run_sweep_chain_delay().await.map_err(|e| {
        let error_context = format!("Chain Delay Sweep failed: {}", e);
        crate::config::ConfigError::ValidationError(error_context)
    })?;
    logging::log("SIMULATOR", "Chain delay sweep completed successfully");
    run_plot("simulator/scripts/sim_sweep_chain_delay/plot_results.py", "Chain Delay Sweep");
    
    // Run duration sweep
    logging::log("SIMULATOR", "=== Running Duration Sweep ===");
    crate::run_sweep_total_block_number().await.map_err(|e| {
        let error_context = format!("Duration Sweep failed: {}", e);
        crate::config::ConfigError::ValidationError(error_context)
    })?;
    logging::log("SIMULATOR", "Duration sweep completed successfully");
    run_plot("simulator/scripts/sim_sweep_total_block_number/plot_results.py", "Duration Sweep");
    
    // Run CAT lifetime sweep
    logging::log("SIMULATOR", "=== Running CAT Lifetime Sweep ===");
    crate::run_sweep_cat_lifetime_simulation().await.map_err(|e| {
        let error_context = format!("CAT Lifetime Sweep failed: {}", e);
        crate::config::ConfigError::ValidationError(error_context)
    })?;
    logging::log("SIMULATOR", "CAT lifetime sweep completed successfully");
    run_plot("simulator/scripts/sim_sweep_cat_lifetime/plot_results.py", "CAT Lifetime Sweep");
    
    // Run block interval constant delay sweep
    logging::log("SIMULATOR", "=== Running Block Interval Constant Delay Sweep ===");
    crate::run_sweep_block_interval_constant_delay().await.map_err(|e| {
        let error_context = format!("Block Interval Constant Delay Sweep failed: {}", e);
        crate::config::ConfigError::ValidationError(error_context)
    })?;
    logging::log("SIMULATOR", "Block interval constant delay sweep completed successfully");
    run_plot("simulator/scripts/sim_sweep_block_interval_constant_delay/plot_results.py", "Block Interval Constant Delay Sweep");
    
    // Run block interval scaled delay sweep
    logging::log("SIMULATOR", "=== Running Block Interval Scaled Delay Sweep ===");
    crate::run_sweep_block_interval_scaled_delay().await.map_err(|e| {
        let error_context = format!("Block Interval Scaled Delay Sweep failed: {}", e);
        crate::config::ConfigError::ValidationError(error_context)
    })?;
    logging::log("SIMULATOR", "Block interval scaled delay sweep completed successfully");
    run_plot("simulator/scripts/sim_sweep_block_interval_scaled_delay/plot_results.py", "Block Interval Scaled Delay Sweep");
    
    // Run CAT pending dependencies sweep
    logging::log("SIMULATOR", "=== Running CAT Pending Dependencies Sweep ===");
    crate::run_sweep_cat_pending_dependencies_simulation().await.map_err(|e| {
        let error_context = format!("CAT Pending Dependencies Sweep failed: {}", e);
        crate::config::ConfigError::ValidationError(error_context)
    })?;
    logging::log("SIMULATOR", "CAT pending dependencies sweep completed successfully");
    run_plot("simulator/scripts/sim_sweep_cat_pending_dependencies/plot_results.py", "CAT Pending Dependencies Sweep");
    
    let total_time = start_time.elapsed();
    logging::log("SIMULATOR", "=== All Tests Completed Successfully ===");
    logging::log("SIMULATOR", &format!("Total execution time: {:.2?}", total_time));
    
    println!("All tests completed successfully!");
    println!("Total execution time: {:.2?}", total_time);
    
    Ok(())
} 