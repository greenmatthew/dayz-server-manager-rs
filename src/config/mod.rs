pub mod mod_entry;
pub mod mods_config;
pub mod server_config;

use std::{fs, path::Path};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result, anyhow};

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

    /// Print configuration summary
    pub fn print_summary(&self, server_install_dir: &str) {
        println!("\n=== Configuration Summary ===");
        println!("Server:");
        println!("  steamcmd_dir: {}", self.server.steamcmd_dir);
        println!("  username: {}", self.server.username);
        println!("  install_dir: {server_install_dir}");
        
        println!("Mods:");
        // Show individual mods if present
        if let Some(mod_list) = &self.mods.mod_list {
            if mod_list.is_empty() {
                println!("  Individual mods: (none)");    
            } else {
                println!("  Individual mods:");
                for (index, mod_entry) in mod_list.iter().enumerate() {
                    println!("    {}. {} ({})", index + 1, mod_entry.name, mod_entry.id);
                }
            }
        }
        
        // Show collection URL if present
        if let Some(collection_url) = &self.mods.mod_collection_url {
            if !collection_url.trim().is_empty() {
                println!("  Collection URL: {collection_url}");
            }
        }
        println!();
    }

    /// Check for configuration file and create if missing
    /// Returns the loaded configuration and prints status messages
    pub fn check_and_load(server_install_dir: &str) -> Result<Self> {
        let found_existing_config = Path::new(CONFIG_FILE).exists();
        
        let config = if found_existing_config {
            println_success("Configuration found", 0);
            Self::load(CONFIG_FILE)?
        } else {
            println_failure("Configuration missing", 0);
            println_step("Creating default configuration", 1);
            
            // Create the default config file using the static save function
            Self::save(CONFIG_FILE, DEFAULT_CONFIG)?;
            
            println_success(&format!("Default configuration created: '{CONFIG_FILE}'"), 1);
            Self::parse(DEFAULT_CONFIG)?
        };

        // Always show the config summary
        config.print_summary(server_install_dir);

        if found_existing_config {
            Ok(config)
        } else {
            println!("⚠️  IMPORTANT: Please edit '{CONFIG_FILE}' before running DZSM again:");
            println!("   1. Set your Steam username (account must own DayZ)");
            println!("   2. Adjust steamcmd_dir path if needed");
            println!("   3. Add any mods you want to the mod_list");
            println!();
            println!("   Note: 'anonymous' login will NOT work - you need a valid Steam account!");
            println!("   You must login to SteamCMD manually once to cache credentials.");
            
            Err(anyhow!(
                "New configuration created - please customize '{}' before running again", 
                CONFIG_FILE
            ))
        }
    }
}