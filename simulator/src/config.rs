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
    #[serde(deserialize_with = "deserialize_durations")]
    pub chain_delays: Vec<Duration>,  // Delay for each chain
    pub block_interval: f64,  // Block interval in seconds
}

#[derive(Debug, Deserialize, Clone)]
pub struct AccountConfig {
    pub initial_balance: i64,
    pub num_accounts: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TransactionConfig {
    pub target_tps: f64,
    pub duration_seconds: u64,
    pub zipf_parameter: f64,
    pub ratio_cats: f64,
    pub cat_lifetime_blocks: u64,  // CAT lifetime in blocks
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
}

#[derive(Debug, Deserialize, Clone)]
pub struct SweepZipfConfig {
    pub network: NetworkConfig,
    #[serde(rename = "accounts")]
    pub num_accounts: AccountConfig,
    pub transactions: TransactionConfig,
    pub sweep: SweepParameters,
}

/// Deserialize durations from a vector of f64 values
/// 
/// # Arguments
/// 
/// * `deserializer` - The deserializer to use
/// 
/// # Returns
fn deserialize_durations<'de, D>(deserializer: D) -> Result<Vec<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let delays: Vec<f64> = Vec::deserialize(deserializer)?;
    Ok(delays.into_iter().map(Duration::from_secs_f64).collect())
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

    fn validate(&self) -> Result<(), ConfigError> {
        if self.num_accounts.initial_balance <= 0 {
            return Err(ConfigError::ValidationError("Initial balance must be positive".into()));
        }
        if self.num_accounts.num_accounts == 0 {
            return Err(ConfigError::ValidationError("Number of accounts must be positive".into()));
        }
        if self.transactions.target_tps <= 0.0 {
            return Err(ConfigError::ValidationError("Target TPS must be positive".into()));
        }
        if self.transactions.duration_seconds == 0 {
            return Err(ConfigError::ValidationError("Duration must be positive".into()));
        }
        if self.transactions.zipf_parameter < 0.0 {
            return Err(ConfigError::ValidationError("Zipf parameter must be non-negative".into()));
        }
        if self.transactions.ratio_cats < 0.0 || self.transactions.ratio_cats > 1.0 {
            return Err(ConfigError::ValidationError("Ratio cats must be between 0 and 1".into()));
        }
        if self.transactions.cat_lifetime_blocks == 0 {
            return Err(ConfigError::ValidationError("CAT lifetime blocks must be positive".into()));
        }
        if self.network.num_chains == 0 {
            return Err(ConfigError::ValidationError("Number of chains must be positive".into()));
        }
        if self.network.chain_delays.len() != self.network.num_chains {
            return Err(ConfigError::ValidationError("Number of chain delays must match number of chains".into()));
        }
        for (i, delay) in self.network.chain_delays.iter().enumerate() {
            if delay.as_secs_f64() < 0.0 {
                return Err(ConfigError::ValidationError(format!("Delay for chain {} must be non-negative", i + 1)));
            }
        }
        if self.network.block_interval <= 0.0 {
            return Err(ConfigError::ValidationError("Block interval must be positive".into()));
        }
        Ok(())
    }

    pub fn get_duration(&self) -> Duration {
        Duration::from_secs(self.transactions.duration_seconds)
    }
}

impl SweepConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.num_accounts.initial_balance <= 0 {
            return Err(ConfigError::ValidationError("Initial balance must be positive".into()));
        }
        if self.num_accounts.num_accounts == 0 {
            return Err(ConfigError::ValidationError("Number of accounts must be positive".into()));
        }
        if self.transactions.target_tps <= 0.0 {
            return Err(ConfigError::ValidationError("Target TPS must be positive".into()));
        }
        if self.transactions.duration_seconds == 0 {
            return Err(ConfigError::ValidationError("Duration must be positive".into()));
        }
        if self.transactions.zipf_parameter < 0.0 {
            return Err(ConfigError::ValidationError("Zipf parameter must be non-negative".into()));
        }
        if self.transactions.ratio_cats < 0.0 || self.transactions.ratio_cats > 1.0 {
            return Err(ConfigError::ValidationError("Ratio cats must be between 0 and 1".into()));
        }
        if self.transactions.cat_lifetime_blocks == 0 {
            return Err(ConfigError::ValidationError("CAT lifetime blocks must be positive".into()));
        }
        if self.network.num_chains == 0 {
            return Err(ConfigError::ValidationError("Number of chains must be positive".into()));
        }
        if self.network.chain_delays.len() != self.network.num_chains {
            return Err(ConfigError::ValidationError("Number of chain delays must match number of chains".into()));
        }
        for (i, delay) in self.network.chain_delays.iter().enumerate() {
            if delay.as_secs_f64() < 0.0 {
                return Err(ConfigError::ValidationError(format!("Delay for chain {} must be non-negative", i + 1)));
            }
        }
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        if self.sweep.cat_rate_step.is_none() && self.sweep.zipf_step.is_none() {
            return Err(ConfigError::ValidationError("Either CAT rate step or Zipf step must be specified".into()));
        }
        if self.network.block_interval <= 0.0 {
            return Err(ConfigError::ValidationError("Block interval must be positive".into()));
        }
        Ok(())
    }

    pub fn get_duration(&self) -> Duration {
        Duration::from_secs(self.transactions.duration_seconds)
    }
}

impl SweepZipfConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.num_accounts.initial_balance <= 0 {
            return Err(ConfigError::ValidationError("Initial balance must be positive".into()));
        }
        if self.num_accounts.num_accounts == 0 {
            return Err(ConfigError::ValidationError("Number of accounts must be positive".into()));
        }
        if self.transactions.target_tps <= 0.0 {
            return Err(ConfigError::ValidationError("Target TPS must be positive".into()));
        }
        if self.transactions.duration_seconds == 0 {
            return Err(ConfigError::ValidationError("Duration must be positive".into()));
        }
        if self.transactions.zipf_parameter < 0.0 {
            return Err(ConfigError::ValidationError("Zipf parameter must be non-negative".into()));
        }
        if self.transactions.ratio_cats < 0.0 || self.transactions.ratio_cats > 1.0 {
            return Err(ConfigError::ValidationError("Ratio cats must be between 0 and 1".into()));
        }
        if self.transactions.cat_lifetime_blocks == 0 {
            return Err(ConfigError::ValidationError("CAT lifetime blocks must be positive".into()));
        }
        if self.network.num_chains == 0 {
            return Err(ConfigError::ValidationError("Number of chains must be positive".into()));
        }
        if self.network.chain_delays.len() != self.network.num_chains {
            return Err(ConfigError::ValidationError("Number of chain delays must match number of chains".into()));
        }
        for (i, delay) in self.network.chain_delays.iter().enumerate() {
            if delay.as_secs_f64() < 0.0 {
                return Err(ConfigError::ValidationError(format!("Delay for chain {} must be non-negative", i + 1)));
            }
        }
        if self.sweep.num_simulations == 0 {
            return Err(ConfigError::ValidationError("Number of simulations must be positive".into()));
        }
        if self.sweep.cat_rate_step.is_none() && self.sweep.zipf_step.is_none() {
            return Err(ConfigError::ValidationError("Either CAT rate step or Zipf step must be specified".into()));
        }
        if self.network.block_interval <= 0.0 {
            return Err(ConfigError::ValidationError("Block interval must be positive".into()));
        }
        Ok(())
    }

    pub fn get_duration(&self) -> Duration {
        Duration::from_secs(self.transactions.duration_seconds)
    }
} 