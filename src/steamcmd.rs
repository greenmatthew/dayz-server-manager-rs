use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

use crate::ui::status::{println_failure, println_step, println_success};
use crate::ui::prompt::prompt_yes_no;

const STEAMCMD_EXE: &str = "steamcmd.exe";
const STEAMCMD_DOWNLOAD_URL: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip";

pub struct SteamCmdManager {
    steamcmd_dir: PathBuf,
}

impl SteamCmdManager {
    pub fn new(steamcmd_dir: &str) -> Self {
        Self {
            steamcmd_dir: PathBuf::from(steamcmd_dir),
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
            return Err(anyhow!(
                "SteamCMD directory does not exist: '{}'\nPlease create the directory or update your config.toml",
                self.steamcmd_dir.display()
            ));
        }

        // Check if directory is empty
        if !self.is_directory_empty()? {
            return Err(anyhow!(
                "SteamCMD directory is not empty: '{}'\nPlease clear the directory or choose a different path in config.toml",
                self.steamcmd_dir.display()
            ));
        }

        // Ask user if they want to install SteamCMD
        println_step(&format!("Would you like to install SteamCMD at: \"{}\"", self.steamcmd_dir.display()), 1);
        
        if !prompt_yes_no("Proceed with installation?", false, 1)? {
            return Err(anyhow!("SteamCMD installation declined by user"));
        }

        self.download_and_install()?;
        println_success("SteamCMD found", 0);
        
        Ok(())
    }

    /// Check if the steamcmd directory is empty
    fn is_directory_empty(&self) -> Result<bool> {
        let entries = fs::read_dir(&self.steamcmd_dir)
            .context("Failed to read SteamCMD directory")?;
        
        Ok(entries.count() == 0)
    }

    /// Download and install SteamCMD
    fn download_and_install(&self) -> Result<()> {
        println_step("Downloading SteamCMD...", 1);
        
        // Download the zip file
        let zip_data = self.download_steamcmd_zip()?;
        
        println_step("Unzipping...", 2);
        
        // Extract the zip file
        self.extract_zip(zip_data)?;
        
        Ok(())
    }

    /// Download SteamCMD zip file
    fn download_steamcmd_zip(&self) -> Result<Vec<u8>> {
        // For now, we'll use a simple approach. In a real implementation,
        // you'd want to use a proper HTTP client like reqwest
        
        // This is a placeholder - you'll need to implement actual HTTP downloading
        // For now, let's return an error with instructions
        Err(anyhow!(
            "HTTP downloading not yet implemented.\n\
            Please manually download SteamCMD from:\n\
            {}\n\
            And extract it to: {}",
            STEAMCMD_DOWNLOAD_URL,
            self.steamcmd_dir.display()
        ))
    }

    /// Extract zip file to steamcmd directory
    fn extract_zip(&self, _zip_data: Vec<u8>) -> Result<()> {
        // This would use a zip extraction library like zip-rs
        // For now, it's a placeholder
        Ok(())
    }

    /// Get the path to steamcmd.exe
    pub fn get_exe_path(&self) -> PathBuf {
        self.steamcmd_dir.join(STEAMCMD_EXE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_steamcmd_found() {
        let temp_dir = TempDir::new().unwrap();
        let steamcmd_dir = temp_dir.path().to_string_lossy();
        
        // Create fake steamcmd.exe
        let steamcmd_exe = temp_dir.path().join(STEAMCMD_EXE);
        fs::write(&steamcmd_exe, "fake exe").unwrap();
        
        let manager = SteamCmdManager::new(&steamcmd_dir);
        assert!(manager.check_and_install().is_ok());
    }

    #[test]
    fn test_directory_not_exists() {
        let manager = SteamCmdManager::new("/nonexistent/path");
        assert!(manager.check_and_install().is_err());
    }

    #[test]
    fn test_directory_not_empty() {
        let temp_dir = TempDir::new().unwrap();
        let steamcmd_dir = temp_dir.path().to_string_lossy();
        
        // Create a file in the directory
        let some_file = temp_dir.path().join("somefile.txt");
        fs::write(some_file, "content").unwrap();
        
        let manager = SteamCmdManager::new(&steamcmd_dir);
        assert!(manager.check_and_install().is_err());
    }

    #[test]
    fn test_is_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SteamCmdManager::new(&temp_dir.path().to_string_lossy());
        
        // Should be empty initially
        assert!(manager.is_directory_empty().unwrap());
        
        // Add a file
        let some_file = temp_dir.path().join("test.txt");
        fs::write(some_file, "test").unwrap();
        
        // Should not be empty now
        assert!(!manager.is_directory_empty().unwrap());
    }
}