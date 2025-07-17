//! Interactive interface system for simulation selection and execution.
//! 
//! Provides user interface for selecting simulation types and managing plot generation.

use std::io::{self, Write};
use std::process::Command;
use std::hash::Hash;
use std::path::Path;

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
    /// CAT pending dependencies sweep
    SweepCatPendingDependencies,
    /// Block interval sweep with all scaled (TPS scaled to maintain constant txs per block)
    SweepBlockIntervalAllScaled,
    /// Block interval sweep with constant time delay
    SweepBlockIntervalConstantTimeDelay,
    /// Block interval sweep with constant block delay
    SweepBlockIntervalConstantBlockDelay,
    /// CAT lifetime parameter sweep
    SweepCatLifetime,
    /// Total block number parameter sweep
    SweepTotalBlockNumber,
    /// Chain delay parameter sweep
    SweepChainDelay,
    /// Zipf distribution parameter sweep
    SweepZipf,
    /// Run all test scenarios
    RunAllTests,
    /// Regenerate all plots
    RunAllPlots,
    /// Run only tests that don't have existing data
    RunMissingTests,
    /// Exit the simulator
    Exit,
}


impl SimulationType {
    /// Converts user input string to simulation type
    pub fn from_input(input: &str) -> Option<Self> {
        match input.trim() {
            "1" => Some(SimulationType::Simple),
            "2" => Some(SimulationType::SweepBlockIntervalAllScaled),
            "3" => Some(SimulationType::SweepBlockIntervalConstantBlockDelay),
            "4" => Some(SimulationType::SweepBlockIntervalConstantTimeDelay),
            "5" => Some(SimulationType::SweepCatLifetime),
            "6" => Some(SimulationType::SweepCatPendingDependencies),
            "7" => Some(SimulationType::SweepCatRate),
            "8" => Some(SimulationType::SweepChainDelay),
            "9" => Some(SimulationType::SweepTotalBlockNumber),
            "10" => Some(SimulationType::SweepZipf),
            "11" => Some(SimulationType::RunAllTests),
            "12" => Some(SimulationType::RunAllPlots),
            "13" => Some(SimulationType::RunMissingTests),
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
        "Available simulation types:\n  1. Simple simulation\n  ------------------------\n  2. Sweep Block Interval (All Scaled)\n  3. Sweep Block Interval (Constant Block Delay)\n  4. Sweep Block Interval (Constant Time Delay)\n  5. Sweep CAT lifetime\n  6. Sweep CAT Pending Dependencies\n  7. Sweep CAT rate\n  8. Sweep Chain Delay\n  9. Sweep Total Block Number\n 10. Sweep Zipf distribution\n  ------------------------\n 11. Run All Tests\n 12. Rerun All Plots Only\n 13. Run Missing Tests Only\n  0. Exit"
    }

    /// Displays the simulator menu
    pub fn show_menu(&self) {
        println!("=== Hyperplane Simulator ===");
        println!("{}", self.get_menu_text());
    }

    /// Gets user choice from input
    pub fn get_user_choice(&self) -> Option<SimulationType> {
        print!("\nSelect simulation type: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        SimulationType::from_input(&input)
    }

    /// Checks if data exists for a specific simulation type
    pub fn data_exists(&self, simulation_type: &str) -> bool {
        let data_path = match simulation_type {
            "simple" => "simulator/results/sim_simple/data",
            "sweep_cat_rate" => "simulator/results/sim_sweep_cat_rate/data",
            "sweep_cat_pending_dependencies" => "simulator/results/sim_sweep_cat_pending_dependencies/data",
            "sweep_block_interval_constant_time_delay" => "simulator/results/sim_sweep_block_interval_constant_time_delay/data",
            "sweep_block_interval_constant_block_delay" => "simulator/results/sim_sweep_block_interval_constant_block_delay/data",
            "sweep_block_interval_all_scaled" => "simulator/results/sim_sweep_block_interval_all_scaled/data",
            "sweep_cat_lifetime" => "simulator/results/sim_sweep_cat_lifetime/data",
            "sweep_total_block_number" => "simulator/results/sim_sweep_total_block_number/data",
            "sweep_chain_delay" => "simulator/results/sim_sweep_chain_delay/data",
            "sweep_zipf" => "simulator/results/sim_sweep_zipf/data",
            _ => return false,
        };
        
        Path::new(data_path).exists()
    }

    /// Runs only simulations that don't have existing data
    pub async fn run_missing_tests(&self) -> Result<(), String> {
        let simulation_types = vec![
            ("simple", "Simple Simulation"),
            ("sweep_cat_rate", "CAT Rate Sweep"),
            ("sweep_cat_pending_dependencies", "CAT Pending Dependencies Sweep"),
            ("sweep_block_interval_constant_time_delay", "Block Interval (Constant Time Delay) Sweep"),
            ("sweep_block_interval_constant_block_delay", "Block Interval (Constant Block Delay) Sweep"),
            ("sweep_block_interval_all_scaled", "Block Interval (All Scaled) Sweep"),
            ("sweep_cat_lifetime", "CAT Lifetime Sweep"),
            ("sweep_total_block_number", "Total Block Number Sweep"),
            ("sweep_chain_delay", "Chain Delay Sweep"),
            ("sweep_zipf", "Zipf Distribution Sweep"),
        ];

        let mut missing_tests = Vec::new();
        
        // Check which tests don't have data
        for (sim_type, name) in &simulation_types {
            if !self.data_exists(sim_type) {
                missing_tests.push((*sim_type, *name));
            }
        }

        if missing_tests.is_empty() {
            println!("All tests have existing data. No tests to run.");
            return Ok(());
        }

        println!("Found {} tests without data:", missing_tests.len());
        for (_, name) in &missing_tests {
            println!("  - {}", name);
        }
        println!();

        // Run each missing test
        for (sim_type, name) in missing_tests {
            println!("Running {}...", name);
            
            // Map simulation type to SimulationType enum
            let simulation_type = match sim_type {
                "simple" => SimulationType::Simple,
                "sweep_cat_rate" => SimulationType::SweepCatRate,
                "sweep_cat_pending_dependencies" => SimulationType::SweepCatPendingDependencies,
                "sweep_block_interval_constant_time_delay" => SimulationType::SweepBlockIntervalConstantTimeDelay,
                "sweep_block_interval_constant_block_delay" => SimulationType::SweepBlockIntervalConstantBlockDelay,
                "sweep_block_interval_all_scaled" => SimulationType::SweepBlockIntervalAllScaled,
                "sweep_cat_lifetime" => SimulationType::SweepCatLifetime,
                "sweep_total_block_number" => SimulationType::SweepTotalBlockNumber,
                "sweep_chain_delay" => SimulationType::SweepChainDelay,
                "sweep_zipf" => SimulationType::SweepZipf,
                _ => continue,
            };

            // Use the registry to run the simulation
            let registry = crate::simulation_registry::get_registry().await;
            let registry_guard = registry.lock().await;
            
            if let Some(config) = registry_guard.get(&simulation_type) {
                // Run the simulation
                let run_future = (config.run_fn)();
                if let Err(e) = run_future.await {
                    return Err(format!("Failed to run {}: {}", name, e));
                }
                
                // Generate plots if a script is specified
                if !config.plot_script.is_empty() {
                    println!("Generating plots for {}...", name);
                    if let Err(e) = self.generate_plots(sim_type) {
                        return Err(format!("Plot generation failed for {}: {}", name, e));
                    }
                }
                
                println!("{} completed successfully!", name);
            } else {
                return Err(format!("Unknown simulation type: {}", sim_type));
            }
        }

        println!("All missing tests completed successfully!");
        Ok(())
    }

    /// Generates plots for a specific simulation type
    pub fn generate_plots(&self, simulation_type: &str) -> Result<(), String> {
        let script_path = match simulation_type {
            "simple" => "simulator/src/scenarios/sim_simple/plot_results.py",

            "sweep_cat_rate" => "simulator/src/scenarios/sim_sweep_cat_rate/plot_results.py",
            "sweep_cat_pending_dependencies" => "simulator/src/scenarios/sim_sweep_cat_pending_dependencies/plot_results.py",
            "sweep_block_interval_constant_time_delay" => "simulator/src/scenarios/sim_sweep_block_interval_constant_time_delay/plot_results.py",
            "sweep_block_interval_constant_block_delay" => "simulator/src/scenarios/sim_sweep_block_interval_constant_block_delay/plot_results.py",
            "sweep_block_interval_all_scaled" => "simulator/src/scenarios/sim_sweep_block_interval_all_scaled/plot_results.py",
            "sweep_cat_lifetime" => "simulator/src/scenarios/sim_sweep_cat_lifetime/plot_results.py",
            "sweep_total_block_number" => "simulator/src/scenarios/sim_sweep_total_block_number/plot_results.py",
            "sweep_chain_delay" => "simulator/src/scenarios/sim_sweep_chain_delay/plot_results.py",
            "sweep_zipf" => "simulator/src/scenarios/sim_sweep_zipf/plot_results.py",
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
                Some(SimulationType::RunMissingTests) => {
                    if let Err(e) = self.run_missing_tests().await {
                        return Err(format!("Missing tests failed: {}", e));
                    }
                    println!("All missing tests completed successfully!");
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
                                SimulationType::SweepBlockIntervalAllScaled => "sweep_block_interval_all_scaled",
                                SimulationType::SweepBlockIntervalConstantBlockDelay => "sweep_block_interval_constant_block_delay",
                                SimulationType::SweepBlockIntervalConstantTimeDelay => "sweep_block_interval_constant_time_delay",
                                SimulationType::SweepCatLifetime => "sweep_cat_lifetime",
                                SimulationType::SweepCatPendingDependencies => "sweep_cat_pending_dependencies",
                                SimulationType::SweepCatRate => "sweep_cat_rate",
                                SimulationType::SweepChainDelay => "sweep_chain_delay",
                                SimulationType::SweepTotalBlockNumber => "sweep_total_block_number",
                                SimulationType::SweepZipf => "sweep_zipf",
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
                    println!("Invalid choice. Please enter a valid choice or 0 to exit.");
                    println!("{}", self.get_menu_text());
                }
            }
        }
        
        Ok(())
    }

    /// Reruns all plot generation scripts
    pub fn rerun_all_plots(&self) -> Result<(), String> {
        let plot_scripts = [
            ("1. Simple Simulation", "sim_simple", "simulator/src/scenarios/sim_simple/plot_results.py"),
            ("2. Sweep Block Interval (All Scaled)", "sweep_block_interval_all_scaled", "simulator/src/scenarios/sim_sweep_block_interval_all_scaled/plot_results.py"),
            ("3. Sweep Block Interval (Constant Block Delay)", "sweep_block_interval_constant_block_delay", "simulator/src/scenarios/sim_sweep_block_interval_constant_block_delay/plot_results.py"),
            ("4. Sweep Block Interval (Constant Time Delay)", "sweep_block_interval_constant_time_delay", "simulator/src/scenarios/sim_sweep_block_interval_constant_time_delay/plot_results.py"),
            ("5. Sweep CAT Lifetime", "sweep_cat_lifetime", "simulator/src/scenarios/sim_sweep_cat_lifetime/plot_results.py"),
            ("6. Sweep CAT Pending Dependencies", "sweep_cat_pending_dependencies", "simulator/src/scenarios/sim_sweep_cat_pending_dependencies/plot_results.py"),
            ("7. Sweep CAT Rate", "sweep_cat_rate", "simulator/src/scenarios/sim_sweep_cat_rate/plot_results.py"),
            ("8. Sweep Chain Delay", "sweep_chain_delay", "simulator/src/scenarios/sim_sweep_chain_delay/plot_results.py"),
            ("9. Sweep Total Block Number", "sweep_total_block_number", "simulator/src/scenarios/sim_sweep_total_block_number/plot_results.py"),
            ("10. Sweep Zipf Distribution", "sweep_zipf", "simulator/src/scenarios/sim_sweep_zipf/plot_results.py"),
        ];
        
        for (title, _name, script) in &plot_scripts {
            println!("\n------------ {} -----------", title);
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