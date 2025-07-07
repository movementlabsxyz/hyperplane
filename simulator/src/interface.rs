use std::io::{self, Write};
use std::process::Command;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationType {
    Simple,
    SweepCatRate,
    SweepZipf,
    SweepChainDelay,
    SweepTotalBlockNumber,
    SweepCatLifetime,
    SweepBlockIntervalConstantBlockDelay,
    SweepBlockIntervalConstantTimeDelay,
    SweepCatPendingDependencies,
    RunAllTests,
    RunAllPlots,
    Exit,
}

impl SimulationType {
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

pub struct SimulatorInterface;

impl SimulatorInterface {
    pub fn new() -> Self {
        Self
    }

    pub fn get_menu_text(&self) -> &'static str {
        "Available simulation types:\n  1. Simple simulation\n  2. Sweep CAT rate\n  3. Sweep Zipf distribution\n  4. Sweep Chain Delay\n  5. Sweep Total Block Number\n  6. Sweep CAT lifetime\n  7. Sweep Block Interval (Constant Block Delay)\n  8. Sweep Block Interval (Constant Time Delay)\n  9. Sweep CAT Pending Dependencies\n 10. Run All Tests\n 11. Rerun All Plots Only\n  0. Exit"
    }

    pub fn show_menu(&self) {
        println!("=== Hyperplane Simulator ===");
        println!("{}", self.get_menu_text());
    }

    pub fn get_user_choice(&self) -> Option<SimulationType> {
        print!("\nSelect simulation type (1-10): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        SimulationType::from_input(&input)
    }

    pub fn generate_plots(&self, simulation_type: &str) -> Result<(), String> {
        // Execute the appropriate plotting script based on simulation type
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