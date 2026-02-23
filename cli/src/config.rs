use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::error::CliError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SssConfig {
    pub rpc_url: String,
    pub keypair_path: String,
    pub stablecoin_preset: u8,
    pub default_decimals: u8,
}

impl Default for SssConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            keypair_path: "~/.config/solana/id.json".to_string(),
            stablecoin_preset: 2,
            default_decimals: 6,
        }
    }
}

pub fn load_config(path: &str) -> Result<SssConfig, CliError> {
    if !Path::new(path).exists() {
        return Ok(SssConfig::default());
    }

    let contents = fs::read_to_string(path)
        .map_err(|e| CliError::ConfigNotFound(e.to_string()))?;
        
    let config: SssConfig = toml::from_str(&contents)
        .map_err(|e| CliError::InvalidConfig(e.to_string()))?;
        
    Ok(config)
}
