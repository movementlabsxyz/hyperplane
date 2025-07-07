//! Central registry for all simulation types in the Hyperplane simulator.
//! Maps simulation types to their configuration and execution logic for easy lookup and deduplication.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::scenarios::{
    sim_simple::run_simple_simulation,
    sim_sweep_cat_rate::run_sweep_cat_rate_simulation,
    sim_sweep_zipf::run_sweep_zipf_simulation,
    sim_sweep_chain_delay::run_sweep_chain_delay,
    sim_sweep_total_block_number::run_sweep_total_block_number,
    sim_sweep_cat_lifetime::run_sweep_cat_lifetime_simulation,
    sim_sweep_block_interval_constant_block_delay::run_sweep_block_interval_constant_block_delay,
    sim_sweep_block_interval_constant_time_delay::run_sweep_block_interval_constant_time_delay,
    sim_sweep_cat_pending_dependencies::run_sweep_cat_pending_dependencies_simulation,
    run_all_tests::run_all_tests,
};

use super::interface::SimulationType;

/// Configuration for a simulation type
pub struct SimulationConfig {
    pub name: &'static str,
    pub run_fn: Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>>>> + Send + Sync>,
    pub plot_script: &'static str,
}

/// Registry that holds all simulation configurations
pub struct SimulationRegistry {
    simulations: HashMap<SimulationType, SimulationConfig>,
}

impl SimulationRegistry {
    pub fn new() -> Self {
        let mut simulations = HashMap::new();
        
        // Register all simulation types
        simulations.insert(SimulationType::Simple, SimulationConfig {
            name: "Simple Simulation",
            run_fn: Box::new(|| Box::pin(async {
                run_simple_simulation().await
                    .map_err(|e| format!("Simple simulation failed: {}", e))
            })),
            plot_script: "simulator/scripts/sim_simple/plot_results.py",
        });
        
        simulations.insert(SimulationType::SweepCatRate, SimulationConfig {
            name: "CAT Rate Sweep",
            run_fn: Box::new(|| Box::pin(async {
                run_sweep_cat_rate_simulation().await
                    .map_err(|e| format!("CAT rate sweep failed: {}", e))
            })),
            plot_script: "simulator/scripts/sim_sweep_cat_rate/plot_results.py",
        });
        
        simulations.insert(SimulationType::SweepZipf, SimulationConfig {
            name: "Zipf Distribution Sweep",
            run_fn: Box::new(|| Box::pin(async {
                run_sweep_zipf_simulation().await
                    .map_err(|e| format!("Zipf sweep failed: {}", e))
            })),
            plot_script: "simulator/scripts/sim_sweep_zipf/plot_results.py",
        });
        
        simulations.insert(SimulationType::SweepChainDelay, SimulationConfig {
            name: "Chain Delay Sweep",
            run_fn: Box::new(|| Box::pin(async {
                run_sweep_chain_delay().await
                    .map_err(|e| format!("Chain delay sweep failed: {}", e))
            })),
            plot_script: "simulator/scripts/sim_sweep_chain_delay/plot_results.py",
        });
        
        simulations.insert(SimulationType::SweepTotalBlockNumber, SimulationConfig {
            name: "Total Block Number Sweep",
            run_fn: Box::new(|| Box::pin(async {
                run_sweep_total_block_number().await
                    .map_err(|e| format!("Total block number sweep failed: {}", e))
            })),
            plot_script: "simulator/scripts/sim_sweep_total_block_number/plot_results.py",
        });
        
        simulations.insert(SimulationType::SweepCatLifetime, SimulationConfig {
            name: "CAT Lifetime Sweep",
            run_fn: Box::new(|| Box::pin(async {
                run_sweep_cat_lifetime_simulation().await
                    .map_err(|e| format!("CAT lifetime sweep failed: {}", e))
            })),
            plot_script: "simulator/scripts/sim_sweep_cat_lifetime/plot_results.py",
        });
        
        simulations.insert(SimulationType::SweepBlockIntervalConstantBlockDelay, SimulationConfig {
            name: "Block Interval (Constant Block Delay) Sweep",
            run_fn: Box::new(|| Box::pin(async {
                run_sweep_block_interval_constant_block_delay().await
                    .map_err(|e| format!("Block interval constant block delay sweep failed: {}", e))
            })),
            plot_script: "simulator/scripts/sim_sweep_block_interval_constant_block_delay/plot_results.py",
        });
        
        simulations.insert(SimulationType::SweepBlockIntervalConstantTimeDelay, SimulationConfig {
            name: "Block Interval (Constant Time Delay) Sweep",
            run_fn: Box::new(|| Box::pin(async {
                run_sweep_block_interval_constant_time_delay().await
                    .map_err(|e| format!("Block interval constant time delay sweep failed: {}", e))
            })),
            plot_script: "simulator/scripts/sim_sweep_block_interval_constant_time_delay/plot_results.py",
        });
        
        simulations.insert(SimulationType::SweepCatPendingDependencies, SimulationConfig {
            name: "CAT Pending Dependencies Sweep",
            run_fn: Box::new(|| Box::pin(async {
                run_sweep_cat_pending_dependencies_simulation().await
                    .map_err(|e| format!("CAT pending dependencies sweep failed: {}", e))
            })),
            plot_script: "simulator/scripts/sim_sweep_cat_pending_dependencies/plot_results.py",
        });
        
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
    
    pub fn get(&self, simulation_type: &SimulationType) -> Option<&SimulationConfig> {
        self.simulations.get(simulation_type)
    }
    
    pub fn get_plot_script(&self, simulation_type: &SimulationType) -> Option<&str> {
        self.simulations.get(simulation_type)
            .map(|config| config.plot_script)
    }
}

// Global registry instance
lazy_static::lazy_static! {
    static ref REGISTRY: Arc<Mutex<SimulationRegistry>> = Arc::new(Mutex::new(SimulationRegistry::new()));
}

/// Get a reference to the global registry
pub async fn get_registry() -> Arc<Mutex<SimulationRegistry>> {
    REGISTRY.clone()
} 