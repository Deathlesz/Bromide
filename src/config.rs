use std::{fs::File, io::Write as _};

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

const CONFIG_PATH: &str = "Config.toml";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    general: GeneralConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeneralConfig {
    ip: String,
    port: u16,
    api_url: String,
}

impl Config {
    pub fn try_load() -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(CONFIG_PATH)?;

        Ok(toml::from_str(&contents)?)
    }

    pub fn try_save(&self) -> Result<(), ConfigError> {
        let serialized = toml::to_string_pretty(self)?;

        Ok(File::create(CONFIG_PATH)?.write_all(serialized.as_bytes())?)
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.general.ip, self.general.port)
    }

    pub fn api_url(&self) -> &str {
        &self.general.api_url
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            ip: "0.0.0.0".into(),
            port: 8080,
            api_url: "/database".into(),
        }
    }
}
