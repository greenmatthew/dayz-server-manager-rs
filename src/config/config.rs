use std::{fs, path::Path};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use super::{server_config::ServerConfig, mods_config::ModsConfig};

const CONFIG_FILE: &str = "config.toml";
const DEFAULT_CONFIG: &str = include_str!("../../defaults/config.toml");

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub mods: ModsConfig,
}

impl Config {
    pub fn load(config_path: &str) -> Result<Self> {
        let config_content = fs::read_to_string(config_path)
            .context("Failed to read config file")?;
        Self::parse(&config_content)
    }

    pub fn parse(raw_toml: &str) -> Result<Self> {
        toml::from_str(raw_toml)
            .context("Failed to parse config")
    }

    pub fn save(&self, config_path: &str) -> Result<()> {
        let config_content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(config_path, config_content)
            .context("Failed to write config file")
    }

    pub fn load_or_create(config_path: &str) -> Result<Self> {
        if Path::new(config_path).exists() {
            Self::load(config_path)
        } else {
            fs::write(config_path, DEFAULT_CONFIG)
                .context("Failed to create default config")?;
            Self::parse(DEFAULT_CONFIG)
        }
    }
}