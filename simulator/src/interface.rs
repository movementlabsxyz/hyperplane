use std::io::{self, Write};
use std::process::Command;

pub enum SimulationType {
    SimpleSim,
    DummySim,
    Exit,
}

impl SimulationType {
    pub fn from_input(input: &str) -> Option<Self> {
        match input.trim() {
            "1" => Some(SimulationType::SimpleSim),
            "2" => Some(SimulationType::DummySim),
            "3" => Some(SimulationType::Exit),
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
        "Available simulation types:\n  1. Simple simulation\n  2. Dummy simulation (not yet implemented)\n  3. Exit"
    }

    pub fn show_menu(&self) {
        println!("=== Hyperplane Simulator ===");
        println!("{}", self.get_menu_text());
    }

    pub fn get_user_choice(&self) -> Option<SimulationType> {
        print!("\nSelect simulation type (1-3): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        SimulationType::from_input(&input)
    }

    pub fn run_dummy_simulation(&self) -> Result<(), String> {
        println!("Dummy simulation is not yet implemented.");
        println!("This will be a placeholder for future simulation types.");
        Ok(())
    }

    pub fn generate_plots(&self) -> Result<(), String> {
        // Execute the simple simulation plotting script
        let output = Command::new("python3")
            .arg("simulator/scripts/simple-sim/plot_results.py")
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
                Some(SimulationType::SimpleSim) => {
                    // Call the async simulation function
                    if let Err(e) = crate::run_simple_simulation().await {
                        return Err(format!("Simulation failed: {}", e));
                    }
                    
                    // Generate plots after successful simulation
                    println!("Generating plots...");
                    if let Err(e) = self.generate_plots() {
                        return Err(format!("Plot generation failed: {}", e));
                    }
                    
                    println!("Simple simulation completed successfully!");
                    break;
                }
                Some(SimulationType::DummySim) => {
                    if let Err(e) = self.run_dummy_simulation() {
                        return Err(format!("Dummy simulation failed: {}", e));
                    }
                    break;
                }
                Some(SimulationType::Exit) => {
                    println!("Exiting...");
                    break;
                }
                None => {
                    println!("Invalid choice. Please enter 1, 2, or 3.");
                    println!("{}", self.get_menu_text());
                }
            }
        }
        
        Ok(())
    }
} 