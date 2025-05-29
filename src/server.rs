use anyhow::{Context, Result, anyhow};
use std::os::windows::fs::{symlink_dir, symlink_file};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::cell::OnceCell;

use crate::config::mod_entry::ModEntry;
use crate::steamcmd::{SteamCmdManager};
use crate::ui::status::{println_step, println_success, println_failure};
use crate::config::Config;

use crate::collection_fetcher::CollectionFetcher;

const SERVER_EXE: &str = "DayZServer_x64.exe";
const SERVER_KEYS: &str = "keys";
const SERVER_CONFIG: &str = "serverDZ.cfg";
const SERVER_PROFILES: &str = "profiles";

pub struct ServerManager {
    config: Config,
    server_install_dir: PathBuf,
    steamcmd_manager: Option<SteamCmdManager>,
    collection_mod_list: OnceCell<Vec<ModEntry>>,
}

impl ServerManager {
    pub fn new(config: Config, server_install_dir: &str) -> Self {
        Self {
            config,
            server_install_dir: PathBuf::from(server_install_dir),
            steamcmd_manager: None,
            collection_mod_list: OnceCell::new(),
        }
    }

    pub fn setup_steamcmd(&mut self) -> Result<()> {  // Make self mutable
        // Handle the Result and extract the value
        let steamcmd = SteamCmdManager::new(&self.config.server.steamcmd_dir)?;
        self.steamcmd_manager = Some(steamcmd);
        Ok(())
    }

    pub fn install_or_update_server(&self) -> Result<()> {
        // Ensure SteamCMD is setup
        if self.steamcmd_manager.is_none() {
            return Err(anyhow!("SteamCMD has not been setup yet."));
        }

        // Get reference to steamcmd manager
        let steamcmd = self.steamcmd_manager.as_ref().unwrap();
        let server_config = &self.config.server;  // Take reference

        steamcmd.install_or_update_app(
            &self.server_install_dir.to_string_lossy(),  // Convert PathBuf to &str
            &server_config.username,
            server_config.server_app_id,
            true
        )?; 

        Ok(())
    }

    pub fn install_or_update_mods(&self) -> Result<()> {
        self.uninstall_prev_mod_installations();

        let individual_mods = self.get_individual_mods();
        let collection_mods = self.get_collection_mods();
        
        // Check if we have any mods to install
        if individual_mods.is_empty() && collection_mods.is_empty() {
            println_success("No mods configured, skipping mod installation", 0);
            return Ok(());
        }

        let mut failed_mods = Vec::new();

        // Install individual mods
        for mod_entry in individual_mods {
            if let Err(e) = self.install_mod(mod_entry.id, &mod_entry.name) {
                println_failure(&format!("Failed to install mod {}: {}", mod_entry.name, e), 0);
                failed_mods.push(mod_entry.name.clone());
            }
        }

        // Install collection mods
        for mod_entry in collection_mods {
            if let Err(e) = self.install_mod(mod_entry.id, &mod_entry.name) {
                println_failure(&format!("Failed to install mod {}: {}", mod_entry.name, e), 0);
                failed_mods.push(mod_entry.name.clone());
            }
        }

        // Report results
        if failed_mods.is_empty() {
            println_success("All mods installed successfully", 0);
        } else {
            println_failure(&format!("Failed to install {} mod(s): {}", 
                failed_mods.len(), 
                failed_mods.join(", ")), 0);
            return Err(anyhow!("Some mods failed to install. Check SteamCMD output above for details."));
        }

        Ok(())
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
        let mut args = vec![format!("-config={SERVER_CONFIG}")];

        args.push(format!("-profiles={SERVER_PROFILES}"));
        
        // Add mods if any are configured
        if let Some(mods_string) = self.build_mods_string() {
            args.push(format!("-mod={mods_string}"));
        }

        // Run the server - this should be interactive like SteamCMD
        self.run_server_with_args(&args)?;
        
        println_success("DayZ server has stopped", 0);
        Ok(())
    }

    /// Clean up all previous mod installations before installing new ones
    fn uninstall_prev_mod_installations(&self) {
        println_step("Cleaning up previous mod installations...", 1);
        
        // Remove all @* directories
        self.cleanup_mod_directories();
        
        // Clear keys directory
        self.cleanup_keys_directory();
        
        println_success("Previous mod installations cleaned up", 2);
    }

    /// Remove all @* directories from server install directory
    fn cleanup_mod_directories(&self) {
        if let Ok(entries) = fs::read_dir(&self.server_install_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('@') {
                        println_step(&format!("Removing: {name}"), 1);
                        let _ = fs::remove_dir_all(&path);
                    }
                }
            }
        }
    }

    /// Remove all contents from keys directory except dayz.bikey
    fn cleanup_keys_directory(&self) {
        let keys_dir = self.server_install_dir.join("keys");
        if keys_dir.exists() {
            println_step("Clearing keys directory (keeping dayz.bikey)...", 2);
            if let Ok(entries) = fs::read_dir(&keys_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        // Skip dayz.bikey (case insensitive)
                        if filename.to_lowercase() != "dayz.bikey" {
                            let _ = fs::remove_file(path);
                        }
                    }
                }
            }
        }
    }

    /// Get individual mods from config
    fn get_individual_mods(&self) -> &[ModEntry] {
        self.config.mods.mod_list.as_deref().unwrap_or(&[])
    }

    /// Get collection mods (cached)
    fn get_collection_mods(&self) -> &[ModEntry] {
        self.collection_mod_list.get_or_init(|| {
            if let Some(collection_url) = &self.config.mods.mod_collection_url {
                if collection_url.trim().is_empty() {
                    Vec::new()
                } else {
                    CollectionFetcher::fetch_collection_mods(collection_url)
                        .unwrap_or_else(|e| {
                            println_failure(&format!("Failed to fetch collection: {e}"), 0);
                            Vec::new()
                        })
                }
            } else {
                Vec::new()
            }
        })
    }

    /// Get all mods combined
    fn get_all_mods(&self) -> Vec<&ModEntry> {
        let mut all_mods = Vec::new();
        all_mods.extend(self.get_individual_mods());
        all_mods.extend(self.get_collection_mods());
        all_mods
    }

    fn install_mod(&self, workshop_id: u64, name: &str) -> Result<()> {
        // Ensure SteamCMD is setup
        if self.steamcmd_manager.is_none() {
            return Err(anyhow!("SteamCMD has not been setup yet."));
        }

        // Get reference to steamcmd manager
        let steamcmd = self.steamcmd_manager.as_ref().unwrap();
        let server_config = &self.config.server;

        println_step(&format!("Attempting to install {name} ({workshop_id})..."), 2);
        println_step("Downloading...", 3);

        let mod_source_path = steamcmd.download_or_update_mod(
            &server_config.username,
            server_config.game_app_id,
            workshop_id,
            true
        )?;

        println_step("Installing...", 4);

        let mod_target_path = self.server_install_dir
            .join(format!("@{name}"));

        if symlink_dir(&mod_source_path, &mod_target_path).is_err() {
            return Err(anyhow!("Failed to create a directory symlink from {mod_source_path:?} to {mod_target_path:?}."));
        }

        // Handle mod keys - symlink individual .bikey files to server keys directory
        let mod_source_keys_path = mod_source_path.join("keys");
        let server_keys_path = self.get_server_keys_path();

        if mod_source_keys_path.exists() {
            println_step("Installing mod keys...", 5);
            
            // Read the keys directory
            match fs::read_dir(&mod_source_keys_path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let key_file_path = entry.path();
                        
                        // Only process .bikey files
                        if let Some(extension) = key_file_path.extension() {
                            if extension.to_string_lossy().to_lowercase() == "bikey" {
                                if let Some(filename) = key_file_path.file_name() {
                                    let target_key_path = server_keys_path.join(filename);
                                    
                                    // Use symlink_file for individual files
                                    if symlink_file(&key_file_path, &target_key_path).is_err() {
                                        return Err(anyhow!(
                                            "Failed to create key file symlink from {key_file_path:?} to {target_key_path:?}"
                                        ));
                                    }
                                    
                                    println_step(&format!("Linked key: {}", filename.to_string_lossy()), 6);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow!(
                        "Failed to read keys directory {mod_source_keys_path:?}: {e}"
                    ));
                }
            }
        } else {
            println_step("No keys required for this mod (client-side or configuration mod)", 5);
        }

        println_success(&format!("Successfully installed {name}"), 2);
        Ok(())
    }

    fn get_server_keys_path(&self) -> PathBuf {
        self.server_install_dir.join(SERVER_KEYS)
    }

    /// Get the full path to the DayZ server executable
    fn get_server_exe_path(&self) -> PathBuf {
        self.server_install_dir.join(SERVER_EXE)
    }

    /// Build the mods string in the format: @ModName1;@ModName2;@ModName3
    fn build_mods_string(&self) -> Option<String> {
        let complete_mod_list = self.get_all_mods();
        if complete_mod_list.is_empty() {
            None
        } else {
            Some(complete_mod_list.iter()
                .map(|mod_entry| format!("@{}", mod_entry.name))
                .collect::<Vec<String>>()
                .join(";"))
        }
    }

    /// Run the DayZ server with arguments, allowing interactive input/output
    fn run_server_with_args(&self, args: &[String]) -> Result<()> {
        let server_exe_path = self.get_server_exe_path();
        
        println_step(&format!("Executing: {} {}", SERVER_EXE, args.join(" ")), 2);
        println!();
        
        // Use spawn() to allow interactive input/output (server console, etc.)
        let mut child = Command::new(&server_exe_path)
            .args(args)
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
