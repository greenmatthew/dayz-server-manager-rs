use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;
use std::io::Cursor;
use curl::easy::Easy;
use std::process::{Command, Stdio};

use crate::ui::status::{println_failure, println_step, println_success};
use crate::ui::prompt::prompt_yes_no;
use crate::config::Config;

const STEAMCMD_EXE: &str = "steamcmd.exe";
const STEAMCMD_DOWNLOAD_URL: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip";

pub struct SteamCmdManager {
    config: Config,
    server_install_dir: PathBuf,
    steamcmd_dir: PathBuf,
}

impl SteamCmdManager {
    pub fn new(config: Config, server_install_dir: &str) -> Self {
        let steamcmd_dir = PathBuf::from(&config.server.steamcmd_dir);
        Self {
            config,
            server_install_dir: PathBuf::from(server_install_dir),
            steamcmd_dir,
        }
    }

    /// Check if SteamCMD is installed and handle installation if needed
    pub fn check_and_install(&self) -> Result<()> {
        let steamcmd_exe_path = self.get_exe_path();

        // Check if steamcmd.exe exists
        if steamcmd_exe_path.exists() {
            println_success("SteamCMD found", 0);
            return Ok(());
        }

        println_failure("SteamCMD missing", 0);

        // Check if directory exists
        if !self.steamcmd_dir.exists() {
            println_step(&format!("Creating SteamCMD directory: {}", self.steamcmd_dir.display()), 1);
            fs::create_dir_all(&self.steamcmd_dir)
                .context("Failed to create SteamCMD directory")?;
        }

        // Check if directory is empty (if it existed)
        if !self.is_directory_empty()? {
            return Err(anyhow!(
                "SteamCMD directory is not empty: '{}'\nPlease clear the directory or choose a different path in config.toml",
                self.steamcmd_dir.display()
            ));
        }

        // Ask user if they want to install SteamCMD
        println_step(&format!("Would you like to install SteamCMD at: \"{}\"", self.steamcmd_dir.display()), 1);
        
        if !prompt_yes_no("Proceed with installation?", true, 1)? {
            return Err(anyhow!("SteamCMD installation declined by user"));
        }

        self.download_and_install()?;
        println_success("SteamCMD installed successfully", 0);
        
        Ok(())
    }

    /// Update the DayZ server (always with validation)
    pub fn update_server(&self) -> Result<()> {
        println_step("Updating DayZ server...", 1);
        
        // Create server directory if it doesn't exist
        if !self.server_install_dir.exists() {
            println_step(&format!("Creating server directory: {}", self.server_install_dir.display()), 2);
            fs::create_dir_all(&self.server_install_dir)
                .context("Failed to create server directory")?;
        }
        
        let install_dir_str = self.server_install_dir.to_string_lossy().to_string();
        let server_app_id_str = self.config.server.server_app_id.to_string();
        
        let args = vec![
            "+force_install_dir",
            &install_dir_str,
            "+login",
            &self.config.server.username,
            "+app_update",
            &server_app_id_str,
            "validate", // Always validate as requested
            "+quit",
        ];
        
        self.run_steamcmd_with_args(args)?;
        
        println!();
        println!();
        println_success("Server update completed", 0);

        Ok(())
    }

    /// Update mods from the configuration
    pub fn update_mods(&self) -> Result<()> {
        if self.config.mods.mod_list.is_empty() {
            println_step("No mods configured, skipping mod updates", 0);
            return Ok(());
        }

        println_step(&format!("Updating {} mod(s)...", self.config.mods.mod_list.len()), 0);
        
        for mod_entry in &self.config.mods.mod_list {
            println_step(&format!("Updating mod: {} ({})", mod_entry.name, mod_entry.id), 1);
            
            let mod_id_str = mod_entry.id.to_string();
            let game_app_id_str = self.config.server.game_app_id.to_string();
            
            let args = vec![
                "+login",
                &self.config.server.username,
                "+workshop_download_item",
                &game_app_id_str,
                &mod_id_str,
                "validate", // Always validate
                "+quit",
            ];
            
            self.run_steamcmd_with_args(args)?;
        }
        
        println_success("Mod updates completed", 0);
        Ok(())
    }

    /// Check if the SteamCMD directory is empty
    fn is_directory_empty(&self) -> Result<bool> {
        let entries = fs::read_dir(&self.steamcmd_dir)
            .context("Failed to read SteamCMD directory")?;
        
        Ok(entries.count() == 0)
    }

    /// Download and install SteamCMD
    fn download_and_install(&self) -> Result<()> {
        println_step("Downloading SteamCMD...", 2);
        
        // Download the zip file
        let zip_data = self.download_steamcmd_zip()?;
        
        println_step("Extracting SteamCMD...", 2);
        
        // Extract the zip file
        self.extract_zip(zip_data)?;
        
        println_success("SteamCMD extraction complete", 2);
        
        Ok(())
    }

    /// Download SteamCMD zip file using curl
    fn download_steamcmd_zip(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut handle = Easy::new();
        
        handle.url(STEAMCMD_DOWNLOAD_URL)?;
        handle.follow_location(true)?;
        handle.timeout(std::time::Duration::from_secs(60))?; // 60 seconds total timeout
        
        {
            let mut transfer = handle.transfer();
            transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;
            transfer.perform()?;
        }
        
        // Check HTTP status
        let response_code = handle.response_code()?;
        if response_code != 200 {
            return Err(anyhow!("HTTP error {}: Failed to download SteamCMD", response_code));
        }
        
        if data.is_empty() {
            return Err(anyhow!("Downloaded file is empty"));
        }
        
        println_success(&format!("Downloaded {} bytes", data.len()), 3);
        Ok(data)
    }

    /// Extract zip file to SteamCMD directory
    fn extract_zip(&self, zip_data: Vec<u8>) -> Result<()> {
        use zip::ZipArchive;
        use std::io::Read;
        
        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)
            .context("Failed to read zip archive")?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to access file in zip")?;
            
            let file_path = self.steamcmd_dir.join(file.name());
            
            // Create parent directories if needed
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create parent directories")?;
            }
            
            // Extract file
            if file.is_dir() {
                fs::create_dir_all(&file_path)
                    .context("Failed to create directory")?;
            } else {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)
                    .context("Failed to read file from zip")?;
                
                fs::write(&file_path, contents)
                    .context("Failed to write extracted file")?;
                
                println_step(&format!("Extracted: {}", file.name()), 3);
            }
        }
        
        Ok(())
    }

    /// Get the path to steamcmd.exe
    pub fn get_exe_path(&self) -> PathBuf {
        self.steamcmd_dir.join(STEAMCMD_EXE)
    }

    /// Run SteamCMD with arguments, allowing interactive input
    fn run_steamcmd_with_args(&self, args: Vec<&str>) -> Result<()> {
        let steamcmd_exe = self.get_exe_path();
        
        println_step(&format!("Running SteamCMD with args: {args:?}"), 2);
        println!();
        
        // Use spawn() instead of output() to allow interactive input
        let mut child = Command::new(&steamcmd_exe)
            .args(&args)
            .stdin(Stdio::inherit())   // Allow user input
            .stdout(Stdio::inherit())  // Show output directly
            .stderr(Stdio::inherit())  // Show errors directly
            .spawn()
            .context("Failed to execute SteamCMD")?;
        
        // Wait for the process to complete
        let status = child.wait()
            .context("Failed to wait for SteamCMD process")?;
        
        if !status.success() {
            return Err(anyhow!(
                "SteamCMD failed with exit code: {:?}", 
                status.code()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServerConfig, ModsConfig};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config() -> Config {
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
    fn test_steamcmd_found() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_config();
        config.server.steamcmd_dir = temp_dir.path().to_string_lossy().to_string();
        
        // Create fake steamcmd.exe
        let steamcmd_exe = temp_dir.path().join(STEAMCMD_EXE);
        fs::write(&steamcmd_exe, "fake exe").unwrap();
        
        let manager = SteamCmdManager::new(config, "test_server");
        assert!(manager.check_and_install().is_ok());
    }

    #[test]
    fn test_is_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_config();
        config.server.steamcmd_dir = temp_dir.path().to_string_lossy().to_string();
        
        let manager = SteamCmdManager::new(config, "test_server");
        
        // Should be empty initially
        assert!(manager.is_directory_empty().unwrap());
        
        // Add a file
        let some_file = temp_dir.path().join("test.txt");
        fs::write(some_file, "test").unwrap();
        
        // Should not be empty now
        assert!(!manager.is_directory_empty().unwrap());
    }
}