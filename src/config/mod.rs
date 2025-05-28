pub mod mod_entry;
pub mod mods_config;
pub mod server_config;

use std::{fs, path::Path};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

pub use server_config::ServerConfig;
pub use mods_config::ModsConfig;

use crate::ui::status::{println_failure, println_step, println_success};

const CONFIG_FILE: &str = "config.toml";
const DEFAULT_CONFIG: &str = include_str!("../../defaults/config.toml");

#[derive(Debug, Deserialize, Serialize, Clone)]
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

    /// Static function to save configuration content to file
    pub fn save(config_path: &str, config_content: &str) -> Result<()> {
        fs::write(config_path, config_content)
            .context("Failed to write config file")
    }

    /// Save this config instance to file (convenience method)
    pub fn _save_to_file(&self, config_path: &str) -> Result<()> {
        let config_content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        Self::save(config_path, &config_content)
    }

    /// Check for configuration file and create if missing
    /// Returns the loaded configuration and prints status messages
    pub fn check_and_load() -> Result<Self> {
        if Path::new(CONFIG_FILE).exists() {
            println_success("Configuration found", 0);
            Self::load(CONFIG_FILE)
        } else {
            println_failure("Configuration missing", 0);
            println_step("Saving default configuration", 1);
            
            // Create the default config file using the static save function
            Self::save(CONFIG_FILE, DEFAULT_CONFIG)?;
            
            println_success("Configuration created", 0);
            Self::parse(DEFAULT_CONFIG)
        }
    }
}