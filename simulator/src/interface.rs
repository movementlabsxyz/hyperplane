//! Interactive interface system for simulation selection and execution.
//! 
//! Provides user interface for selecting simulation types and managing plot generation.

use std::io::{self, Write};
use std::process::Command;
use std::hash::Hash;

// ------------------------------------------------------------------------------------------------
// Simulation Type Enum
// ------------------------------------------------------------------------------------------------

/// Available simulation types for the Hyperplane simulator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationType {
    /// Simple simulation with default parameters
    Simple,
    /// CAT rate parameter sweep
    SweepCatRate,
    /// Zipf distribution parameter sweep
    SweepZipf,
    /// Chain delay parameter sweep
    SweepChainDelay,
    /// Total block number parameter sweep
    SweepTotalBlockNumber,
    /// CAT lifetime parameter sweep
    SweepCatLifetime,
    /// Block interval sweep with constant block delay
    SweepBlockIntervalConstantBlockDelay,
    /// Block interval sweep with constant time delay
    SweepBlockIntervalConstantTimeDelay,
    /// CAT pending dependencies sweep
    SweepCatPendingDependencies,
    /// Run all test scenarios
    RunAllTests,
    /// Regenerate all plots
    RunAllPlots,
    /// Exit the simulator
    Exit,
}


impl SimulationType {
    /// Converts user input string to simulation type
    pub fn from_input(input: &str) -> Option<Self> {
        match input.trim() {
            "1" => Some(SimulationType::Simple),
            "2" => Some(SimulationType::SweepCatRate),
            "3" => Some(SimulationType::SweepZipf),
            "4" => Some(SimulationType::SweepChainDelay),
            "5" => Some(SimulationType::SweepTotalBlockNumber),
            "6" => Some(SimulationType::SweepCatLifetime),
            "7" => Some(SimulationType::SweepBlockIntervalConstantBlockDelay),
            "8" => Some(SimulationType::SweepBlockIntervalConstantTimeDelay),
            "9" => Some(SimulationType::SweepCatPendingDependencies),
            "10" => Some(SimulationType::RunAllTests),
            "11" => Some(SimulationType::RunAllPlots),
            "0" => Some(SimulationType::Exit),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Simulator Interface
// ------------------------------------------------------------------------------------------------

/// Main interface for user interaction with the simulator
pub struct SimulatorInterface;

impl SimulatorInterface {
    /// Creates a new simulator interface
    pub fn new() -> Self {
        Self
    }

    /// Returns the menu text for available simulation types
    pub fn get_menu_text(&self) -> &'static str {
        "Available simulation types:\n  1. Simple simulation\n  2. Sweep CAT rate\n  3. Sweep Zipf distribution\n  4. Sweep Chain Delay\n  5. Sweep Total Block Number\n  6. Sweep CAT lifetime\n  7. Sweep Block Interval (Constant Block Delay)\n  8. Sweep Block Interval (Constant Time Delay)\n  9. Sweep CAT Pending Dependencies\n 10. Run All Tests\n 11. Rerun All Plots Only\n  0. Exit"
    }

    /// Displays the simulator menu
    pub fn show_menu(&self) {
        println!("=== Hyperplane Simulator ===");
        println!("{}", self.get_menu_text());
    }

    /// Gets user choice from input
    pub fn get_user_choice(&self) -> Option<SimulationType> {
        print!("\nSelect simulation type (1-10): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        SimulationType::from_input(&input)
    }

    /// Generates plots for a specific simulation type
    pub fn generate_plots(&self, simulation_type: &str) -> Result<(), String> {
        let script_path = match simulation_type {
            "simple" => "simulator/scripts/sim_simple/plot_results.py",
            "sweep_cat_rate" => "simulator/scripts/sim_sweep_cat_rate/plot_results.py",
            "sweep_zipf" => "simulator/scripts/sim_sweep_zipf/plot_results.py",
            "sweep_chain_delay" => "simulator/scripts/sim_sweep_chain_delay/plot_results.py",
            "sweep_total_block_number" => "simulator/scripts/sim_sweep_total_block_number/plot_results.py",
            "sweep_cat_lifetime" => "simulator/scripts/sim_sweep_cat_lifetime/plot_results.py",
            "sweep_block_interval_constant_block_delay" => "simulator/scripts/sim_sweep_block_interval_constant_block_delay/plot_results.py",
            "sweep_block_interval_constant_time_delay" => "simulator/scripts/sim_sweep_block_interval_constant_time_delay/plot_results.py",
            "sweep_cat_pending_dependencies" => "simulator/scripts/sim_sweep_cat_pending_dependencies/plot_results.py",
            _ => return Err(format!("Unknown simulation type: {}", simulation_type)),
        };

        let output = Command::new("python3")
            .arg(script_path)
            .output()
            .map_err(|e| format!("Failed to execute plotting script: {}", e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Plot generation failed: {}", error_msg));
        }

        if !output.stdout.is_empty() {
            println!("Plot output: {}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }

    /// Main simulation loop with user interaction
    pub async fn run_simple_simulation_async(&self) -> Result<(), String> {
        loop {
            self.show_menu();
            
            match self.get_user_choice() {
                Some(SimulationType::Exit) => {
                    println!("Exiting...");
                    break;
                }
                Some(SimulationType::RunAllPlots) => {
                    if let Err(e) = self.rerun_all_plots() {
                        return Err(format!("Plot rerun failed: {}", e));
                    }
                    println!("All plot scripts rerun successfully!");
                    break;
                }
                Some(simulation_type) => {
                    // Use the registry to run the simulation
                    let registry = crate::simulation_registry::get_registry().await;
                    let registry_guard = registry.lock().await;
                    
                    if let Some(config) = registry_guard.get(&simulation_type) {
                        println!("Running {}...", config.name);
                        
                        // Run the simulation
                        let run_future = (config.run_fn)();
                        if let Err(e) = run_future.await {
                            return Err(e);
                        }

                        // Generate plots if a script is specified
                        if !config.plot_script.is_empty() {
                            println!("Generating plots...");
                            let plot_type = match simulation_type {
                                SimulationType::Simple => "simple",
                                SimulationType::SweepCatRate => "sweep_cat_rate",
                                SimulationType::SweepZipf => "sweep_zipf",
                                SimulationType::SweepChainDelay => "sweep_chain_delay",
                                SimulationType::SweepTotalBlockNumber => "sweep_total_block_number",
                                SimulationType::SweepCatLifetime => "sweep_cat_lifetime",
                                SimulationType::SweepBlockIntervalConstantBlockDelay => "sweep_block_interval_constant_block_delay",
                                SimulationType::SweepBlockIntervalConstantTimeDelay => "sweep_block_interval_constant_time_delay",
                                SimulationType::SweepCatPendingDependencies => "sweep_cat_pending_dependencies",
                                _ => "unknown",
                            };
                            
                            if let Err(e) = self.generate_plots(plot_type) {
                                return Err(format!("Plot generation failed: {}", e));
                            }
                        }
                        
                        println!("{} completed successfully!", config.name);
                        break;
                    } else {
                        return Err(format!("Unknown simulation type: {:?}", simulation_type));
                    }
                }
                None => {
                    println!("Invalid choice. Please enter 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, or 0 to exit.");
                    println!("{}", self.get_menu_text());
                }
            }
        }
        
        Ok(())
    }

    /// Reruns all plot generation scripts
    pub fn rerun_all_plots(&self) -> Result<(), String> {
        let plot_scripts = [
            ("sim_simple", "simulator/scripts/sim_simple/plot_results.py"),
            ("sweep_cat_rate", "simulator/scripts/sim_sweep_cat_rate/plot_results.py"),
            ("sweep_zipf", "simulator/scripts/sim_sweep_zipf/plot_results.py"),
            ("sweep_chain_delay", "simulator/scripts/sim_sweep_chain_delay/plot_results.py"),
            ("sweep_total_block_number", "simulator/scripts/sim_sweep_total_block_number/plot_results.py"),
            ("sweep_cat_lifetime", "simulator/scripts/sim_sweep_cat_lifetime/plot_results.py"),
            ("sweep_block_interval_constant_block_delay", "simulator/scripts/sim_sweep_block_interval_constant_block_delay/plot_results.py"),
            ("sweep_block_interval_constant_time_delay", "simulator/scripts/sim_sweep_block_interval_constant_time_delay/plot_results.py"),
            ("sweep_cat_pending_dependencies", "simulator/scripts/sim_sweep_cat_pending_dependencies/plot_results.py"),
        ];
        
        for (name, script) in &plot_scripts {
            println!("Running plot script for {}...", name);
            let status = Command::new("python3")
                .arg(script)
                .status()
                .map_err(|e| format!("Failed to run {}: {}", script, e))?;
            if !status.success() {
                return Err(format!("Plot script {} failed with status {}", script, status));
            }
        }
        Ok(())
    }
} 