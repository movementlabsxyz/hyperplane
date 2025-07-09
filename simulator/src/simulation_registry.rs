//! Central registry for all simulation types.
//! 
//! Maps simulation types to their configuration and execution logic for easy lookup and deduplication.
//! 
//! See the README.md for instructions on how to add new simulations to this registry.


use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::scenarios::{
    sim_simple,
    sim_sweep_cat_rate,
    sim_sweep_zipf,
    sim_sweep_chain_delay,
    sim_sweep_total_block_number,
    sim_sweep_cat_lifetime,
    sim_sweep_block_interval_constant_block_delay,
    sim_sweep_block_interval_constant_time_delay,
    sim_sweep_cat_pending_dependencies,
    run_all_tests::run_all_tests,
};

use super::interface::SimulationType;

// ------------------------------------------------------------------------------------------------
// Data Structures
// ------------------------------------------------------------------------------------------------

/// Configuration for a simulation type
/// 
/// This struct contains all the information needed to run and manage a simulation:
/// - `name`: Human-readable name for the simulation
/// - `run_fn`: Function that executes the simulation asynchronously
/// - `plot_script`: Path to the Python script that generates plots from results
pub struct SimulationConfig {
    pub name: &'static str,
    pub run_fn: Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>>>> + Send + Sync>,
    pub plot_script: &'static str,
}

/// Registry that holds all simulation configurations
/// 
/// This registry provides a centralized way to manage all available simulations.
/// It maps simulation types to their configurations and provides methods to
/// retrieve simulation information and execute simulations.
pub struct SimulationRegistry {
    simulations: HashMap<SimulationType, SimulationConfig>,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl SimulationRegistry {
    /// Creates a new simulation registry with all available simulations registered.
    /// 
    /// This method initializes the registry by calling the `register()` function
    /// from each simulation module. Each simulation provides its own registration
    /// information, making the registry extensible and maintainable.
    pub fn new() -> Self {
        let mut simulations = HashMap::new();
        
        // Register all simulation types using their register functions
        let (sim_type, sim_config) = sim_simple::simulation::register();
        simulations.insert(sim_type, sim_config);
        
        let (sim_type, sim_config) = sim_sweep_cat_rate::simulation::register();
        simulations.insert(sim_type, sim_config);
        
        let (sim_type, sim_config) = sim_sweep_zipf::simulation::register();
        simulations.insert(sim_type, sim_config);
        
        let (sim_type, sim_config) = sim_sweep_chain_delay::simulation::register();
        simulations.insert(sim_type, sim_config);
        
        let (sim_type, sim_config) = sim_sweep_total_block_number::simulation::register();
        simulations.insert(sim_type, sim_config);
        
        let (sim_type, sim_config) = sim_sweep_cat_lifetime::simulation::register();
        simulations.insert(sim_type, sim_config);
        
        let (sim_type, sim_config) = sim_sweep_block_interval_constant_block_delay::simulation::register();
        simulations.insert(sim_type, sim_config);
        
        let (sim_type, sim_config) = sim_sweep_block_interval_constant_time_delay::simulation::register();
        simulations.insert(sim_type, sim_config);
        
        let (sim_type, sim_config) = sim_sweep_cat_pending_dependencies::simulation::register();
        simulations.insert(sim_type, sim_config);
        
        // Register run all tests (still hardcoded since it doesn't have a register function)
        simulations.insert(SimulationType::RunAllTests, SimulationConfig {
            name: "All Tests",
            run_fn: Box::new(|| Box::pin(async {
                run_all_tests().await
                    .map_err(|e| format!("All tests failed: {}", e))
            })),
            plot_script: "", // No plot script for run all tests
        });
        
        Self { simulations }
    }
    
    /// Retrieves the configuration for a specific simulation type.
    /// 
    /// Returns `None` if the simulation type is not registered.
    pub fn get(&self, simulation_type: &SimulationType) -> Option<&SimulationConfig> {
        self.simulations.get(simulation_type)
    }
    
    /// Retrieves the plot script path for a specific simulation type.
    /// 
    /// Returns `None` if the simulation type is not registered or has no plot script.
    pub fn get_plot_script(&self, simulation_type: &SimulationType) -> Option<&str> {
        self.simulations.get(simulation_type)
            .map(|config| config.plot_script)
    }
}

// ------------------------------------------------------------------------------------------------
// Global Registry Instance
// ------------------------------------------------------------------------------------------------

// Global registry instance
lazy_static::lazy_static! {
    static ref REGISTRY: Arc<Mutex<SimulationRegistry>> = Arc::new(Mutex::new(SimulationRegistry::new()));
}

/// Get a reference to the global registry
pub async fn get_registry() -> Arc<Mutex<SimulationRegistry>> {
    REGISTRY.clone()
} 