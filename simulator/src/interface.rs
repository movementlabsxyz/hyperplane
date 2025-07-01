use std::io::{self, Write};
use std::process::Command;

pub enum SimulationType {
    Simple,
    SweepCatRate,
    SweepZipf,
    SweepChainDelay,
    SweepDuration,
    SweepCatLifetime,
    SweepBlockIntervalConstantDelay,
    SweepBlockIntervalScaledDelay,
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
            "5" => Some(SimulationType::SweepDuration),
            "6" => Some(SimulationType::SweepCatLifetime),
            "7" => Some(SimulationType::SweepBlockIntervalConstantDelay),
            "8" => Some(SimulationType::SweepBlockIntervalScaledDelay),
            "9" => Some(SimulationType::RunAllTests),
            "10" => Some(SimulationType::RunAllPlots),
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
        "Available simulation types:\n  1. Simple simulation\n  2. Sweep CAT rate\n  3. Sweep Zipf distribution\n  4. Sweep Chain Delay\n  5. Sweep Duration\n  6. Sweep CAT lifetime\n  7. Sweep Block Interval (Constant Delay)\n  8. Sweep Block Interval (Scaled Delay)\n  9. Run All Tests\n 10. Rerun All Plots Only\n  0. Exit"
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
            "sweep_duration" => "simulator/scripts/sim_sweep_duration/plot_results.py",
            "sweep_cat_lifetime" => "simulator/scripts/sim_sweep_cat_lifetime/plot_results.py",
            "sweep_block_interval_constant_delay" => "simulator/scripts/sim_sweep_block_interval_constant_delay/plot_results.py",
            "sweep_block_interval_scaled_delay" => "simulator/scripts/sim_sweep_block_interval_scaled_delay/plot_results.py",
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
                Some(SimulationType::Simple) => {
                    // Call the async simulation function
                    if let Err(e) = crate::run_simple_simulation().await {
                        return Err(format!("Simulation failed: {}", e));
                    }
                    
                    // Generate plots after successful simulation
                    println!("Generating plots...");
                    if let Err(e) = self.generate_plots("simple") {
                        return Err(format!("Plot generation failed: {}", e));
                    }
                    
                    println!("Simple simulation completed successfully!");
                    break;
                }
                Some(SimulationType::SweepCatRate) => {
                    // Call the sweep simulation function
                    if let Err(e) = crate::run_sweep_cat_rate_simulation().await {
                        return Err(format!("Sweep simulation failed: {}", e));
                    }
                    
                    // Generate plots after successful simulation
                    println!("Generating plots...");
                    if let Err(e) = self.generate_plots("sweep_cat_rate") {
                        return Err(format!("Plot generation failed: {}", e));
                    }
                    
                    println!("Sweep CAT rate simulation completed successfully!");
                    break;
                }
                Some(SimulationType::SweepZipf) => {
                    // Call the sweep Zipf simulation function
                    if let Err(e) = crate::run_sweep_zipf_simulation().await {
                        return Err(format!("Sweep Zipf simulation failed: {}", e));
                    }
                    
                    // Generate plots after successful simulation
                    println!("Generating plots...");
                    if let Err(e) = self.generate_plots("sweep_zipf") {
                        return Err(format!("Plot generation failed: {}", e));
                    }
                    
                    println!("Sweep Zipf distribution simulation completed successfully!");
                    break;
                }
                Some(SimulationType::SweepChainDelay) => {
                    // Call the sweep chain delay simulation function
                    if let Err(e) = crate::run_sweep_chain_delay().await {
                        return Err(format!("Sweep Chain Delay simulation failed: {}", e));
                    }
                    
                    // Generate plots after successful simulation
                    println!("Generating plots...");
                    if let Err(e) = self.generate_plots("sweep_chain_delay") {
                        return Err(format!("Plot generation failed: {}", e));
                    }
                    
                    println!("Sweep Chain Delay simulation completed successfully!");
                    break;
                }
                Some(SimulationType::SweepDuration) => {
                    // Call the sweep duration simulation function
                    if let Err(e) = crate::run_sweep_duration().await {
                        return Err(format!("Sweep Duration simulation failed: {}", e));
                    }
                    
                    // Generate plots after successful simulation
                    println!("Generating plots...");
                    if let Err(e) = self.generate_plots("sweep_duration") {
                        return Err(format!("Plot generation failed: {}", e));
                    }
                    
                    println!("Sweep Duration simulation completed successfully!");
                    break;
                }
                Some(SimulationType::SweepCatLifetime) => {
                    // Call the sweep CAT lifetime simulation function
                    if let Err(e) = crate::run_sweep_cat_lifetime_simulation().await {
                        return Err(format!("Sweep CAT lifetime simulation failed: {}", e));
                    }
                    
                    // Generate plots after successful simulation
                    println!("Generating plots...");
                    if let Err(e) = self.generate_plots("sweep_cat_lifetime") {
                        return Err(format!("Plot generation failed: {}", e));
                    }
                    
                    println!("Sweep CAT lifetime simulation completed successfully!");
                    break;
                }
                Some(SimulationType::SweepBlockIntervalConstantDelay) => {
                    // Call the sweep block interval constant delay simulation function
                    if let Err(e) = crate::run_sweep_block_interval_constant_delay().await {
                        return Err(format!("Sweep Block Interval Constant Delay simulation failed: {}", e));
                    }
                    
                    // Generate plots after successful simulation
                    println!("Generating plots...");
                    if let Err(e) = self.generate_plots("sweep_block_interval_constant_delay") {
                        return Err(format!("Plot generation failed: {}", e));
                    }
                    
                    println!("Sweep Block Interval Constant Delay simulation completed successfully!");
                    break;
                }
                Some(SimulationType::SweepBlockIntervalScaledDelay) => {
                    // Call the sweep block interval scaled delay simulation function
                    if let Err(e) = crate::run_sweep_block_interval_scaled_delay().await {
                        return Err(format!("Sweep Block Interval Scaled Delay simulation failed: {}", e));
                    }
                    
                    // Generate plots after successful simulation
                    println!("Generating plots...");
                    if let Err(e) = self.generate_plots("sweep_block_interval_scaled_delay") {
                        return Err(format!("Plot generation failed: {}", e));
                    }
                    
                    println!("Sweep Block Interval Scaled Delay simulation completed successfully!");
                    break;
                }
                Some(SimulationType::RunAllTests) => {
                    // Call the run all tests function
                    if let Err(e) = crate::scenarios::run_all_tests::run_all_tests().await {
                        return Err(format!("Run All Tests failed: {}", e));
                    }
                    break;
                }
                Some(SimulationType::RunAllPlots) => {
                    if let Err(e) = self.rerun_all_plots() {
                        return Err(format!("Plot rerun failed: {}", e));
                    }
                    println!("All plot scripts rerun successfully!");
                    break;
                }
                Some(SimulationType::Exit) => {
                    println!("Exiting...");
                    break;
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
            ("sweep_duration", "simulator/scripts/sim_sweep_duration/plot_results.py"),
            ("sweep_cat_lifetime", "simulator/scripts/sim_sweep_cat_lifetime/plot_results.py"),
            ("sweep_block_interval_constant_delay", "simulator/scripts/sim_sweep_block_interval_constant_delay/plot_results.py"),
            ("sweep_block_interval_scaled_delay", "simulator/scripts/sim_sweep_block_interval_scaled_delay/plot_results.py"),
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