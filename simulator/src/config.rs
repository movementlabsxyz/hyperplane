use serde::Deserialize;
use std::fs;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub initial_balance: i64,
    pub num_accounts: usize,
    pub target_tps: f64,
    pub duration_seconds: u64,
    pub zipf_parameter: f64,
    pub ratio_cats: f64,
    pub block_interval: f64,  // Block interval in seconds
    pub cat_lifetime: u64,    // Number of blocks a CAT can be pending, before timing out
    pub chains: ChainConfig,
}

#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    pub num_chains: usize,
    #[serde(deserialize_with = "deserialize_durations")]
    pub delays: Vec<Duration>,  // Delay for each chain
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

impl ChainConfig {
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
        let config_str = fs::read_to_string("simulator/config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.initial_balance <= 0 {
            return Err(ConfigError::ValidationError("Initial balance must be positive".into()));
        }
        if self.num_accounts == 0 {
            return Err(ConfigError::ValidationError("Number of accounts must be positive".into()));
        }
        if self.target_tps <= 0.0 {
            return Err(ConfigError::ValidationError("Target TPS must be positive".into()));
        }
        if self.duration_seconds == 0 {
            return Err(ConfigError::ValidationError("Duration must be positive".into()));
        }
        if self.zipf_parameter < 0.0 {
            return Err(ConfigError::ValidationError("Zipf parameter must be non-negative".into()));
        }
        if self.ratio_cats <= 0.0 {
            return Err(ConfigError::ValidationError("Ratio cats must be positive".into()));
        }
        if self.block_interval <= 0.0 {
            return Err(ConfigError::ValidationError("Block interval must be positive".into()));
        }
        if self.cat_lifetime == 0 {
            return Err(ConfigError::ValidationError("CAT lifetime must be positive".into()));
        }
        if self.chains.num_chains == 0 {
            return Err(ConfigError::ValidationError("Number of chains must be positive".into()));
        }
        if self.chains.delays.len() != self.chains.num_chains {
            return Err(ConfigError::ValidationError("Number of chain delays must match number of chains".into()));
        }
        for (i, delay) in self.chains.delays.iter().enumerate() {
            if delay.as_secs_f64() < 0.0 {
                return Err(ConfigError::ValidationError(format!("Delay for chain {} must be non-negative", i + 1)));
            }
        }
        Ok(())
    }

    pub fn get_duration(&self) -> Duration {
        Duration::from_secs(self.duration_seconds)
    }
} 