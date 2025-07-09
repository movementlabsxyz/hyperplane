use std::process::Command;
use hyperplane::utils::logging;
use crate::config::ConfigError;

/// Runs a simulation function with automatic plotting
pub async fn run_simulation_with_plotting<F, Fut>(
    simulation_fn: F,
    simulation_name: &str,
    plot_script_path: &str
) -> Result<(), ConfigError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), ConfigError>>,
{
    logging::log("SIMULATOR", &format!("=== Running {} ===", simulation_name));
    
    // Run the simulation
    simulation_fn().await.map_err(|e| {
        let error_context = format!("{} failed: {}", simulation_name, e);
        ConfigError::ValidationError(error_context)
    })?;
    
    logging::log("SIMULATOR", &format!("{} completed successfully", simulation_name));
    
    // Generate plots
    logging::log("PLOT", &format!("Generating plots for {}...", simulation_name));
    let output = Command::new("python3")
        .arg(plot_script_path)
        .output();
    match output {
        Ok(output) if output.status.success() => {
            logging::log("PLOT", &format!("Plots for {} generated successfully!", simulation_name));
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr);
            logging::log("PLOT", &format!("Plot generation failed for {}: {}", simulation_name, err));
        }
        Err(e) => {
            logging::log("PLOT", &format!("Failed to execute plotting script for {}: {}", simulation_name, e));
        }
    }
    
    Ok(())
} 