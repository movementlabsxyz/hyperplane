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
    pub chains: ChainConfig,
}

#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    pub num_chains: usize,
    pub delays: Vec<f64>,  // Delay in seconds for each chain
}

impl ChainConfig {
    pub fn get_chain_ids(&self) -> Vec<String> {
        (1..=self.num_chains)
            .map(|i| format!("chain-{}", i))
            .collect()
    }

    pub fn get_chain_delay(&self, chain_index: usize) -> Duration {
        Duration::from_secs_f64(self.delays[chain_index])
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
        if self.chains.num_chains == 0 {
            return Err(ConfigError::ValidationError("Number of chains must be positive".into()));
        }
        if self.chains.delays.len() != self.chains.num_chains {
            return Err(ConfigError::ValidationError("Number of chain delays must match number of chains".into()));
        }
        for (i, delay) in self.chains.delays.iter().enumerate() {
            if *delay < 0.0 {
                return Err(ConfigError::ValidationError(format!("Delay for chain {} must be non-negative", i + 1)));
            }
        }
        Ok(())
    }

    pub fn get_duration(&self) -> Duration {
        Duration::from_secs(self.duration_seconds)
    }
} 