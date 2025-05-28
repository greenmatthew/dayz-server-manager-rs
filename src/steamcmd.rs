use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Write, Cursor};
use curl::easy::Easy;

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

    /// Check if the steamcmd directory is empty
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

    /// Extract zip file to steamcmd directory
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