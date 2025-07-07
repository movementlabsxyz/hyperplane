//! Configuration loader and validator for the Hyperplane simulator.
//! Handles parsing, validation, and access to simulation configuration files.


use serde::Deserialize;
use std::fs;
use std::time::Duration;
use thiserror::Error;

// ------------------------------------------------------------------------------------------------
// Main Configuration Structs
// ------------------------------------------------------------------------------------------------

/// Main configuration struct for simulation parameters.
/// 
/// This struct contains all the configuration needed to run a simulation,
/// including network settings, account configuration, and transaction parameters.
/// It is used for both simple simulations and as the base configuration for sweep simulations.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Network configuration including chain count, delays, and block intervals
    pub network_config: NetworkConfig,
    /// Account configuration including initial balances and account count
    pub account_config: AccountConfig,
    /// Transaction configuration including rates, patterns, and cross-chain settings
    pub transaction_config: TransactionConfig,
}

/// Configuration for network-related simulation parameters.
/// 
/// This struct defines the multi-chain network topology and timing characteristics
/// for the simulation, including the number of chains, inter-chain delays, and
/// block production rates.
#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    /// Number of chains in the multi-chain network
    pub num_chains: usize,
    /// Delay in blocks for each chain (order corresponds to chain-1, chain-2, etc.)
    pub chain_delays: Vec<u64>,
    /// Block interval in seconds (time between block productions)
    pub block_interval: f64,
}

/// Configuration for account-related simulation parameters.
/// 
/// This struct defines the account setup for the simulation, including
/// the number of accounts to create and their initial token balances.
#[derive(Debug, Deserialize, Clone)]
pub struct AccountConfig {
    /// Initial balance for each account in the simulation (in tokens)
    pub initial_balance: i64,
    /// Number of accounts to create in the simulation
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

// ------------------------------------------------------------------------------------------------
// Sweep Configuration Structs
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct SweepConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

/// Configuration for sweep simulation parameters.
/// 
/// This struct defines the parameters used to control parameter sweep simulations.
/// Only ONE step parameter should be specified per sweep type - the simulator will
/// generate a sequence of values based on the step size and number of simulations.
#[derive(Debug, Deserialize, Clone)]
pub struct SweepParameters {
    /// Total number of simulation runs in the sweep (determines how many parameter values to test)
    pub num_simulations: usize,
    /// Step size for CAT ratio sweeps (0.0 = no CATs, 1.0 = all CATs)
    #[serde(default)]
    pub cat_rate_step: Option<f64>,
    /// Step size for Zipf distribution parameter sweeps (controls account access pattern skewness)
    #[serde(default)]
    pub zipf_step: Option<f64>,
    /// Step size for chain delay sweeps (in blocks, affects inter-chain communication timing)
    #[serde(default)]
    pub chain_delay_step: Option<f64>,
    /// Step size for simulation duration sweeps (in blocks, affects total simulation length)
    #[serde(default)]
    pub duration_step: Option<u64>,
    /// Step size for CAT lifetime sweeps (in blocks, affects CAT timeout behavior)
    #[serde(default)]
    pub cat_lifetime_step: Option<u64>,
    /// Step size for block interval sweeps (in seconds, affects block production rate)
    #[serde(default)]
    pub block_interval_step: Option<f64>,
    /// Reference delay duration for block interval sweeps (in seconds, used with block_interval_step)
    #[serde(default)]
    pub reference_chain_delay_duration: Option<f64>,
}

// Separate sweep config structs to maintain sweep-specific validation
#[derive(Debug, Deserialize, Clone)]
pub struct SweepZipfConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepChainDelayConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepDurationConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepCatLifetimeConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepBlockIntervalConstantDelayConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepBlockIntervalScaledDelayConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepCatPendingDependenciesConfig {
    pub network_config: NetworkConfig,
    pub account_config: AccountConfig,
    pub transaction_config: TransactionConfig,
    pub sweep: SweepParameters,
}

impl NetworkConfig {
    pub fn get_chain_ids(&self) -> Vec<String> {
        (1..=self.num_chains)
            .map(|i| format!("chain-{}", i))
            .collect()
    }
}

// ------------------------------------------------------------------------------------------------
// Error Types and Validation
// ------------------------------------------------------------------------------------------------

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
pub trait ValidateConfig {
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
    account_config: &AccountConfig,
    transaction_config: &TransactionConfig,
    network_config: &NetworkConfig,
) -> Result<(), ConfigError> {
    if account_config.initial_balance <= 0 {
        return Err(ConfigError::ValidationError("Initial balance must be positive".into()));
    }
    if account_config.num_accounts == 0 {
        return Err(ConfigError::ValidationError("Number of accounts must be positive".into()));
    }
    if transaction_config.target_tps <= 0.0 {
        return Err(ConfigError::ValidationError("Target TPS must be positive".into()));
    }
    if transaction_config.sim_total_block_number == 0 {
        return Err(ConfigError::ValidationError("Simulation total block number must be positive".into()));
    }
    if transaction_config.zipf_parameter < 0.0 {
        return Err(ConfigError::ValidationError("Zipf parameter must be non-negative".into()));
    }
    if transaction_config.ratio_cats < 0.0 || transaction_config.ratio_cats > 1.0 {
        return Err(ConfigError::ValidationError("Ratio cats must be between 0 and 1".into()));
    }
    if transaction_config.cat_lifetime_blocks == 0 {
        return Err(ConfigError::ValidationError("CAT lifetime blocks must be positive".into()));
    }
    if transaction_config.initialization_wait_blocks == 0 {
        return Err(ConfigError::ValidationError("Initialization wait blocks must be positive".into()));
    }
    // allow_cat_pending_dependencies is a boolean, so no validation needed
    if network_config.num_chains == 0 {
        return Err(ConfigError::ValidationError("Number of chains must be positive".into()));
    }
    if network_config.chain_delays.len() != network_config.num_chains {
        return Err(ConfigError::ValidationError("Number of chain delays must match number of chains".into()));
    }
    // No validation needed for u64 - it's always non-negative
    if network_config.block_interval <= 0.0 {
        return Err(ConfigError::ValidationError("Block interval must be positive".into()));
    }
    Ok(())
}

// ------------------------------------------------------------------------------------------------
// Configuration Implementation Methods
// ------------------------------------------------------------------------------------------------

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













    fn validate(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)
    }

    pub fn get_duration(&self) -> Duration {
        // Calculate duration based on block interval and total blocks
        // This is a rough estimate for backward compatibility
        let block_interval = self.network_config.block_interval;
        let total_blocks = self.transaction_config.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

impl ValidateConfig for SweepConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
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
        let block_interval = self.network_config.block_interval;
        let total_blocks = self.transaction_config.sim_total_block_number;
        Duration::from_secs_f64(block_interval * total_blocks as f64)
    }
}

// Implement ValidateConfig for each sweep config type to maintain sweep-specific validation
impl ValidateConfig for SweepZipfConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        Ok(())
    }

    fn validate_sweep_specific(&self) -> Result<(), ConfigError> {
        if self.sweep.zipf_step.is_none() {
            return Err(ConfigError::ValidationError("Zipf step must be specified".into()));
        }
        Ok(())
    }
}

impl ValidateConfig for SweepChainDelayConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
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

impl ValidateConfig for SweepDurationConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
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

impl ValidateConfig for SweepCatLifetimeConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
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

impl ValidateConfig for SweepBlockIntervalConstantDelayConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
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

impl ValidateConfig for SweepBlockIntervalScaledDelayConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
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

impl ValidateConfig for SweepCatPendingDependenciesConfig {
    fn validate_common(&self) -> Result<(), ConfigError> {
        validate_common_fields(&self.account_config, &self.transaction_config, &self.network_config)?;
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