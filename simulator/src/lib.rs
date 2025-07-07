pub mod account_selection;
pub mod zipf_account_selection;
pub mod run_simulation;
pub mod simulation_results;
pub mod network;
pub mod config;
pub mod logging;
pub mod testnodes;
pub mod interface;
pub mod scenarios;
pub mod stats;
pub mod simulation_registry;

pub use account_selection::AccountSelectionStats;
pub use zipf_account_selection::AccountSelector;
pub use run_simulation::run_simulation;
pub use simulation_results::SimulationResults;
pub use network::initialize_accounts;
pub use testnodes::*; 
pub use interface::{SimulatorInterface, SimulationType};
pub use scenarios::sim_simple::run_simple_simulation;
pub use scenarios::sim_sweep_cat_rate::run_sweep_cat_rate_simulation;
pub use scenarios::sim_sweep_zipf::run_sweep_zipf_simulation;
pub use scenarios::sim_sweep_chain_delay::run_sweep_chain_delay;
pub use scenarios::sim_sweep_total_block_number::run_sweep_total_block_number;
pub use scenarios::sim_sweep_cat_lifetime::run_sweep_cat_lifetime_simulation;
pub use scenarios::sim_sweep_block_interval_constant_block_delay::run_sweep_block_interval_constant_block_delay;
pub use scenarios::sim_sweep_block_interval_constant_time_delay::run_sweep_block_interval_constant_time_delay;
pub use scenarios::sim_sweep_cat_pending_dependencies::run_sweep_cat_pending_dependencies_simulation;
pub use scenarios::run_all_tests; 

// ============================================================================
// Flexible Sweep Configuration Macro
// ============================================================================

#[macro_export]
macro_rules! define_sweep_config {
    (
        $config_name:ident,
        $toml_file:expr,
        validate_sweep_specific = $validate_block:expr
    ) => {
        #[derive(Debug, serde::Deserialize, Clone)]
        pub struct $config_name {
            pub network_config: crate::config::NetworkConfig,
            pub account_config: crate::config::AccountConfig,
            pub transaction_config: crate::config::TransactionConfig,
            pub sweep: crate::config::SweepParameters,
        }

        impl crate::config::ValidateConfig for $config_name {
            fn validate_common(&self) -> Result<(), crate::config::ConfigError> {
                crate::config::validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
                if self.sweep.num_simulations == 0 {
                    return Err(crate::config::ConfigError::ValidationError("Number of simulations must be positive".into()));
                }
                Ok(())
            }
            fn validate_sweep_specific(&self) -> Result<(), crate::config::ConfigError> {
                ($validate_block)(self)
            }
        }

        impl crate::scenarios::sweep_runner::SweepConfigTrait for $config_name {
            fn as_any(&self) -> &dyn std::any::Any { self }
            fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
            fn get_network_config(&self) -> &crate::config::NetworkConfig { &self.network_config }
            fn get_account_config(&self) -> &crate::config::AccountConfig { &self.account_config }
            fn get_transaction_config(&self) -> &crate::config::TransactionConfig { &self.transaction_config }
        }

        fn load_config() -> Result<$config_name, crate::config::ConfigError> {
            use std::fs;
            use toml;
            let config_str = fs::read_to_string(concat!("simulator/src/scenarios/", $toml_file))?;
            let config: $config_name = toml::from_str(&config_str)?;
            config.validate()?;
            Ok(config)
        }
    };
} 