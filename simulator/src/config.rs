use serde::Deserialize;
use std::fs;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub simulation: SimulationConfig,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub num_chains: usize,
}

#[derive(Debug, Deserialize)]
pub struct SimulationConfig {
    pub initial_balance: i64,
    pub num_accounts: usize,
    pub target_tps: f64,
    pub duration_seconds: u64,
    pub block_interval_seconds: f64,
    pub zipf_parameter: f64,
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
        if self.network.num_chains == 0 {
            return Err(ConfigError::ValidationError("Number of chains must be positive".into()));
        }
        if self.simulation.initial_balance <= 0 {
            return Err(ConfigError::ValidationError("Initial balance must be positive".into()));
        }
        if self.simulation.num_accounts == 0 {
            return Err(ConfigError::ValidationError("Number of accounts must be positive".into()));
        }
        if self.simulation.target_tps <= 0.0 {
            return Err(ConfigError::ValidationError("Target TPS must be positive".into()));
        }
        if self.simulation.duration_seconds == 0 {
            return Err(ConfigError::ValidationError("Duration must be positive".into()));
        }
        if self.simulation.block_interval_seconds <= 0.0 {
            return Err(ConfigError::ValidationError("Block interval must be positive".into()));
        }
        if self.simulation.zipf_parameter <= 0.0 {
            return Err(ConfigError::ValidationError("Zipf parameter must be positive".into()));
        }
        Ok(())
    }

    pub fn get_duration(&self) -> Duration {
        Duration::from_secs(self.simulation.duration_seconds)
    }
} 