use simulator::{
    interface::SimulatorInterface,
};

// ------------------------------------------------------------------------------------------------
// Main
// ------------------------------------------------------------------------------------------------

/// Main function that orchestrates the simulation setup and execution
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interface = SimulatorInterface::new();
    
    if let Err(e) = interface.run_simple_simulation_async().await {
        eprintln!("Error: {}", e);
    }
    
    Ok(())
}