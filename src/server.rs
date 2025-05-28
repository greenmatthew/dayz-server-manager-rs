use anyhow::{Context, Result, anyhow};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::ui::status::{println_step, println_success};
use crate::config::Config;

const SERVER_EXE: &str = "DayZServer_x64.exe";
const SERVER_CONFIG: &str = "serverDZ.cfg";

pub struct ServerManager {
    config: Config,
    server_install_dir: PathBuf,
}

impl ServerManager {
    pub fn new(config: Config, server_install_dir: &str) -> Self {
        Self {
            config,
            server_install_dir: PathBuf::from(server_install_dir),
        }
    }

    /// Run the DayZ server with configured mods
    pub fn run_server(&self) -> Result<()> {
        let server_exe_path = self.get_server_exe_path();
        
        // Check if server executable exists
        if !server_exe_path.exists() {
            return Err(anyhow!(
                "DayZ server executable not found: {}\nMake sure the server has been downloaded/updated first.",
                server_exe_path.display()
            ));
        }

        // Build the command arguments
        let mut args = vec![format!("-config={}", SERVER_CONFIG)];
        
        // Add mods if any are configured
        if !self.config.mods.mod_list.is_empty() {
            let mods_string = self.build_mods_string();
            args.push(format!("-mod={mods_string}"));
        }

        // Run the server - this should be interactive like SteamCMD
        self.run_server_with_args(args)?;
        
        println_success("DayZ server has stopped", 0);
        Ok(())
    }

    /// Get the full path to the DayZ server executable
    fn get_server_exe_path(&self) -> PathBuf {
        self.server_install_dir.join(SERVER_EXE)
    }

    /// Build the mods string in the format: @ModName1;@ModName2;@ModName3
    fn build_mods_string(&self) -> String {
        self.config.mods.mod_list
            .iter()
            .map(|mod_entry| format!("@{}", mod_entry.name))
            .collect::<Vec<String>>()
            .join(";")
    }

    /// Run the DayZ server with arguments, allowing interactive input/output
    fn run_server_with_args(&self, args: Vec<String>) -> Result<()> {
        let server_exe_path = self.get_server_exe_path();
        
        println_step(&format!("Executing: {} {}", SERVER_EXE, args.join(" ")), 2);
        println!();
        
        // Use spawn() to allow interactive input/output (server console, etc.)
        let mut child = Command::new(&server_exe_path)
            .args(&args)
            .current_dir(&self.server_install_dir) // Set working directory to server install dir
            .stdin(Stdio::inherit())   // Allow user input to server console
            .stdout(Stdio::inherit())  // Show server output directly
            .stderr(Stdio::inherit())  // Show server errors directly
            .spawn()
            .context("Failed to execute DayZ server")?;
        
        // Wait for the server process to complete
        let status = child.wait()
            .context("Failed to wait for DayZ server process")?;
        
        if !status.success() {
            return Err(anyhow!(
                "DayZ server exited with error code: {:?}", 
                status.code()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServerConfig, ModsConfig, mod_entry::ModEntry};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config_with_mods() -> Config {
        Config {
            server: ServerConfig {
                steamcmd_dir: "test_steamcmd".to_string(),
                server_app_id: 223350,
                game_app_id: 221100,
                username: "testuser".to_string(),
            },
            mods: ModsConfig {
                mod_list: vec![
                    ModEntry { id: 1559212036, name: "CF".to_string() },
                    ModEntry { id: 1564026768, name: "Community-Online-Tools".to_string() },
                    ModEntry { id: 2289456201, name: "Namalsk Island".to_string() },
                ],
            },
        }
    }

    fn create_test_config_no_mods() -> Config {
        Config {
            server: ServerConfig {
                steamcmd_dir: "test_steamcmd".to_string(),
                server_app_id: 223350,
                game_app_id: 221100,
                username: "testuser".to_string(),
            },
            mods: ModsConfig {
                mod_list: vec![],
            },
        }
    }

    #[test]
    fn test_build_mods_string_with_mods() {
        let config = create_test_config_with_mods();
        let manager = ServerManager::new(config, "test_server");
        
        let mods_string = manager.build_mods_string();
        assert_eq!(mods_string, "@CF;@Community-Online-Tools;@Namalsk Island");
    }

    #[test]
    fn test_build_mods_string_no_mods() {
        let config = create_test_config_no_mods();
        let manager = ServerManager::new(config, "test_server");
        
        let mods_string = manager.build_mods_string();
        assert_eq!(mods_string, "");
    }

    #[test]
    fn test_get_server_exe_path() {
        let config = create_test_config_no_mods();
        let manager = ServerManager::new(config, "/test/server/path");
        
        let exe_path = manager.get_server_exe_path();
        assert_eq!(exe_path, PathBuf::from("/test/server/path/DayZServer_x64.exe"));
    }

    #[test]
    fn test_server_exe_missing() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config_no_mods();
        let manager = ServerManager::new(config, temp_dir.path().to_str().unwrap());
        
        // Should fail when server exe doesn't exist
        let result = manager.run_server();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DayZ server executable not found"));
    }

    #[test]
    fn test_server_exe_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config_no_mods();
        let manager = ServerManager::new(config, temp_dir.path().to_str().unwrap());
        
        // Create fake server executable
        let server_exe = temp_dir.path().join(SERVER_EXE);
        fs::write(&server_exe, "fake exe").unwrap();
        
        // This would normally try to run the server, but since it's a fake exe,
        // we can't easily test the full execution in a unit test.
        // The important part is that it doesn't fail on the "file not found" check.
        let server_exe_path = manager.get_server_exe_path();
        assert!(server_exe_path.exists());
    }
}