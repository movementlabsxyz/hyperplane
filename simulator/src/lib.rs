//! Hyperplane Simulator Library
//! 
//! Simulation framework for testing the Hyperplane protocol under various conditions.
//! Supports simple simulations, parameter sweeps, and performance analysis.

// ------------------------------------------------------------------------------------------------
// Module Declarations
// ------------------------------------------------------------------------------------------------

/// Account selection statistics tracking and analysis
pub mod account_selection;

/// Zipf distribution-based account selection for realistic transaction patterns
pub mod zipf_account_selection;

/// Core simulation logic and transaction processing
pub mod run_simulation;

/// Simulation results tracking, data collection, and analysis
pub mod simulation_results;

/// Multi-chain network setup, node management, and chain registration
pub mod network;

/// Configuration management, validation, and parameter handling
pub mod config;

/// Logging utilities for simulation output and debugging
pub mod logging;

/// Test node setup and management for simulation environment
pub mod testnodes;

/// Interactive interface system for simulation selection and execution
pub mod interface;

/// Simulation scenarios including simple simulations and parameter sweeps
pub mod scenarios;

/// Performance statistics collection and analysis
pub mod stats;

/// Central registry for all simulation types and configurations
pub mod simulation_registry;

// ------------------------------------------------------------------------------------------------
// Public Exports
// ------------------------------------------------------------------------------------------------

// Core simulation components
pub use run_simulation::run_simulation;
pub use simulation_results::SimulationResults;
pub use interface::{SimulatorInterface, SimulationType};

// Configuration and network management
pub use config::Config;


// Account selection and statistics
pub use account_selection::AccountSelectionStats;
pub use zipf_account_selection::AccountSelector;

// Test utilities
pub use testnodes::*; 

// Simple simulation
pub use scenarios::sim_simple::simulation::run_simple_simulation;

// Sweep simulations
pub use scenarios::sim_sweep_cat_rate::simulation::run_sweep_cat_rate_simulation;
pub use scenarios::sim_sweep_zipf::simulation::run_sweep_zipf_simulation;
pub use scenarios::sim_sweep_chain_delay::simulation::run_sweep_chain_delay;
pub use scenarios::sim_sweep_total_block_number::simulation::run_sweep_total_block_number;
pub use scenarios::sim_sweep_cat_lifetime::simulation::run_sweep_cat_lifetime_simulation;
pub use scenarios::sim_sweep_block_interval_constant_block_delay::simulation::run_sweep_block_interval_constant_block_delay;
pub use scenarios::sim_sweep_block_interval_constant_time_delay::simulation::run_sweep_block_interval_constant_time_delay;
pub use scenarios::sim_sweep_block_interval_all_scaled::simulation::run_sweep_block_interval_all_scaled;
pub use scenarios::sim_sweep_cat_pending_dependencies::simulation::run_sweep_cat_pending_dependencies_simulation;

// Test orchestration
pub use scenarios::run_all_tests; 