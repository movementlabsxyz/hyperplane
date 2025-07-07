//! Configuration loader and validator for the Hyperplane simulator.
//! Handles parsing, validation, and access to simulation configuration files.

use serde::Deserialize;
use std::fs;
use std::time::Duration;
use thiserror::Error;


#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub num_chains: usize,
    pub chain_delays: Vec<u64>,  // Delay for each chain in blocks
    pub block_interval: f64,  // Block interval in seconds
}

#[derive(Debug, Deserialize, Clone)]
pub struct AccountConfig {
    pub initial_balance: i64,
    pub num_accounts: usize,
}

/// Configuration for transaction-related simulation parameters.
/// 
/// This struct contains all the parameters that control how transactions are generated,
/// processed, and managed during the simulation. It includes settings for transaction
/// rates, simulation duration, account access patterns, and cross-chain transaction behavior.
#[derive(Debug, Deserialize, Clone)]
pub struct TransactionConfig {
    /// Target transactions per second for the simulation (controls transaction generation rate)
    pub target_tps: f64,
    /// Total number of blocks to simulate (determines simulation duration)
    pub sim_total_block_number: u64,
    /// Zipf distribution parameter α (controls account access pattern skewness: 0.0 = uniform, higher = more skewed)
    pub zipf_parameter: f64,
    /// Ratio of Cross-Chain Atomic Transactions (CATs) to regular transactions (0.0 = no CATs, 1.0 = all CATs)
    pub ratio_cats: f64,
    /// Maximum number of blocks a CAT can remain pending before timing out
    pub cat_lifetime_blocks: u64,
    /// Number of blocks to wait after account initialization before starting transaction submission
    pub initialization_wait_blocks: u64,
    /// Whether CATs can depend on locked keys from pending transactions (affects transaction ordering)
    pub allow_cat_pending_dependencies: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepConfig {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepParameters {
    pub num_simulations: usize,
    #[serde(default)]
    pub cat_rate_step: Option<f64>,
    #[serde(default)]
    pub zipf_step: Option<f64>,
    #[serde(default)]
    pub chain_delay_step: Option<f64>,
    #[serde(default)]
    pub duration_step: Option<u64>,
    #[serde(default)]
    pub cat_lifetime_step: Option<u64>,
    #[serde(default)]
    pub block_interval_step: Option<f64>,
    #[serde(default)]
    pub reference_chain_delay_duration: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepZipfConfig {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepChainDelayConfig {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepDurationConfig {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepCatLifetimeConfig {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepBlockIntervalConstantDelayConfig {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepBlockIntervalScaledDelayConfig {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepCatPendingDependenciesConfig {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
    pub sweep: SweepParameters,
}



impl NetworkConfig {
    pub fn get_chain_ids(&self) -> Vec<String> {
        (1..=self.num_chains)
            .map(|i| format!("chain-{}", i))
            .collect()
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Invalid configuration: {0}")]
    ValidationError(String),
}

// Trait for common validation functionality
trait ValidateConfig {
    fn validate_common(&self) -> Result<(), ConfigError>;
    fn validate_sweep_specific(&self) -> Result<(), ConfigError>;
    
    fn validate(&self) -> Result<(), ConfigError> {
        self.validate_common()?;
        self.validate_sweep_specific()?;
        Ok(())
    }
}

// Common validation logic
fn validate_common_fields(
    num_accounts: &AccountConfig,
    transactions: &TransactionConfig,
    network: &NetworkConfig,
) -> Result<(), ConfigError> {
    if num_accounts.initial_balance <= 0 {
        return Err(ConfigError::ValidationError("Initial balance must be positive".into()));
    }
    if num_accounts.num_accounts == 0 {
        return Err(ConfigError::ValidationError("Number of accounts must be positive".into()));
    }
    if transactions.target_tps <= 0.0 {
        return Err(ConfigError::ValidationError("Target TPS must be positive".into()));
    }
    if transactions.sim_total_block_number == 0 {
        return Err(ConfigError::ValidationError("Simulation total block number must be positive".into()));
    }
    if transactions.zipf_parameter < 0.0 {
        return Err(ConfigError::ValidationError("Zipf parameter must be non-negative".into()));
    }
    if transactions.ratio_cats < 0.0 || transactions.ratio_cats > 1.0 {
        return Err(ConfigError::ValidationError("Ratio cats must be between 0 and 1".into()));
    }
    if transactions.cat_lifetime_blocks == 0 {
        return Err(ConfigError::ValidationError("CAT lifetime blocks must be positive".into()));
    }
    if transactions.initialization_wait_blocks == 0 {
        return Err(ConfigError::ValidationError("Initialization wait blocks must be positive".into()));
    }
    // allow_cat_pending_dependencies is a boolean, so no validation needed
    if network.num_chains == 0 {
        return Err(ConfigError::ValidationError("Number of chains must be positive".into()));
    }
    if network.chain_delays.len() != network.num_chains {
        return Err(ConfigError::ValidationError("Number of chain delays must match number of chains".into()));
    }
    // No validation needed for u64 - it's always non-negative
    if network.block_interval <= 0.0 {
        return Err(ConfigError::ValidationError("Block interval must be positive".into()));
    }
    Ok(())
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_str = fs::read_to_string("simulator/src/scenarios/config_simple.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_sweep() -> Result<SweepConfig, ConfigError> {
        let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_cat_rate.toml")?;
        let config: SweepConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_sweep_zipf() -> Result<SweepZipfConfig, ConfigError> {
        let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_zipf.toml")?;
        let config: SweepZipfConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_sweep_chain_delay() -> Result<SweepChainDelayConfig, ConfigError> {
        let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_chain_delay.toml")?;
        let config: SweepChainDelayConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_sweep_total_block_number() -> Result<SweepDurationConfig, ConfigError> {
        let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_total_block_number.toml")?;
        let config: SweepDurationConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_sweep_cat_lifetime() -> Result<SweepCatLifetimeConfig, ConfigError> {
        let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_cat_lifetime.toml")?;
        let config: SweepCatLifetimeConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_sweep_block_interval_constant_block_delay() -> Result<SweepBlockIntervalConstantDelayConfig, ConfigError> {
        let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_block_interval_constant_block_delay.toml")?;
        let config: SweepBlockIntervalConstantDelayConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_sweep_block_interval_constant_time_delay() -> Result<SweepBlockIntervalScaledDelayConfig, ConfigError> {
        let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_block_interval_constant_time_delay.toml")?;
        let config: SweepBlockIntervalScaledDelayConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_sweep_cat_pending_dependencies() -> Result<SweepCatPendingDependenciesConfig, ConfigError> {
        let config_str = fs::read_to_string("simulator/src/scenarios/config_sweep_cat_pending_dependencies.toml")?;
        let config: SweepCatPendingDependenciesConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.num_accounts, &self.transactions, &self.network)
    }

    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network.block_interval;
        let total_blocks = self.transactions.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl ValidateConfig for SweepConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.num_accounts, &self.transactions, &self.network)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        if self.sweep.cat_rate_step.is_none() && self.sweep.zipf_step.is_none() {
            return Err(ConfigError::ValidationError("Either CAT rate step or Zipf step must be specified".into()));
        }
        Ok(())
    }
}

impl SweepConfig {
    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network.block_interval;
        let total_blocks = self.transactions.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl ValidateConfig for SweepZipfConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.num_accounts, &self.transactions, &self.network)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        if self.sweep.cat_rate_step.is_none() && self.sweep.zipf_step.is_none() {
            return Err(ConfigError::ValidationError("Either CAT rate step or Zipf step must be specified".into()));
        }
        Ok(())
    }
}

impl SweepZipfConfig {
    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network.block_interval;
        let total_blocks = self.transactions.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl ValidateConfig for SweepChainDelayConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.num_accounts, &self.transactions, &self.network)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        if self.sweep.chain_delay_step.is_none() {
            return Err(ConfigError::ValidationError("Chain delay step must be specified".into()));
        }
        Ok(())
    }
}

impl SweepChainDelayConfig {
    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network.block_interval;
        let total_blocks = self.transactions.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl ValidateConfig for SweepDurationConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.num_accounts, &self.transactions, &self.network)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        if self.sweep.duration_step.is_none() {
            return Err(ConfigError::ValidationError("Duration step must be specified".into()));
        }
        Ok(())
    }
}

impl SweepDurationConfig {
    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network.block_interval;
        let total_blocks = self.transactions.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl ValidateConfig for SweepCatLifetimeConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.num_accounts, &self.transactions, &self.network)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        if self.sweep.cat_lifetime_step.is_none() {
            return Err(ConfigError::ValidationError("CAT lifetime step must be specified".into()));
        }
        Ok(())
    }
}

impl SweepCatLifetimeConfig {
    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network.block_interval;
        let total_blocks = self.transactions.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl ValidateConfig for SweepBlockIntervalConstantDelayConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.num_accounts, &self.transactions, &self.network)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        if self.sweep.block_interval_step.is_none() {
            return Err(ConfigError::ValidationError("Block interval step must be specified".into()));
        }
        Ok(())
    }
}

impl SweepBlockIntervalConstantDelayConfig {
    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network.block_interval;
        let total_blocks = self.transactions.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl ValidateConfig for SweepBlockIntervalScaledDelayConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.num_accounts, &self.transactions, &self.network)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        if self.sweep.block_interval_step.is_none() {
            return Err(ConfigError::ValidationError("Block interval step must be specified".into()));
        }
        if let Some(ref_delay) = self.sweep.reference_chain_delay_duration {
            if ref_delay <= 0.0 {
                return Err(ConfigError::ValidationError("Reference chain delay duration must be positive".into()));
            }
        }
        Ok(())
    }
}

impl SweepBlockIntervalScaledDelayConfig {
    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network.block_interval;
        let total_blocks = self.transactions.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl crate::scenarios::sweep_runner::SweepConfigTrait for SweepBlockIntervalConstantDelayConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network(&self) -> &crate::config::NetworkConfig { &self.network }
    fn get_num_accounts(&self) -> &crate::config::AccountConfig { &self.num_accounts }
    fn get_transactions(&self) -> &crate::config::TransactionConfig { &self.transactions }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl crate::scenarios::sweep_runner::SweepConfigTrait for SweepBlockIntervalScaledDelayConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network(&self) -> &crate::config::NetworkConfig { &self.network }
    fn get_num_accounts(&self) -> &crate::config::AccountConfig { &self.num_accounts }
    fn get_transactions(&self) -> &crate::config::TransactionConfig { &self.transactions }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl ValidateConfig for SweepCatPendingDependenciesConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.num_accounts, &self.transactions, &self.network)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        // For this sweep, we only need to validate that num_simulations is set
        // The sweep will test exactly 2 values: false and true
        if self.sweep.num_simulations != 2 {
            return Err(ConfigError::ValidationError("Number of simulations must be exactly 2 for CAT pending dependencies sweep (false and true)".into()));
        }
        Ok(())
    }
}

impl SweepCatPendingDependenciesConfig {
    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network.block_interval;
        let total_blocks = self.transactions.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl crate::scenarios::sweep_runner::SweepConfigTrait for SweepCatPendingDependenciesConfig {
    fn get_num_simulations(&self) -> usize { self.sweep.num_simulations }
    fn get_network(&self) -> &crate::config::NetworkConfig { &self.network }
    fn get_num_accounts(&self) -> &crate::config::AccountConfig { &self.num_accounts }
    fn get_transactions(&self) -> &crate::config::TransactionConfig { &self.transactions }
    fn as_any(&self) -> &dyn std::any::Any { self }
} 