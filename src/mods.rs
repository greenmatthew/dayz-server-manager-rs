use anyhow::{Context, Result, anyhow};
use std::fs;
use std::os::windows::fs::symlink_dir;
use std::os::windows::fs::symlink_file;
use std::path::PathBuf;

use crate::ui::status::{println_step, println_success, println_failure};
use crate::config::Config;

pub struct ModsManager {
    config: Config,
    server_install_dir: PathBuf,
    steamcmd_dir: PathBuf,
}

impl ModsManager {
    pub fn new(config: Config, server_install_dir: &str) -> Self {
        let steamcmd_dir = PathBuf::from(&config.server.steamcmd_dir);
        Self {
            config,
            server_install_dir: PathBuf::from(server_install_dir),
            steamcmd_dir,
        }
    }

    /// Install all mods from the configuration
    pub fn install_mods(&self) -> Result<()> {
        if self.config.mods.mod_list.is_empty() {
            println_success("No mods configured, skipping mod installation", 0);
            return Ok(());
        }

        println_step(&format!("Installing {} mod(s)...", self.config.mods.mod_list.len()), 1);

        // Ensure server keys directory exists
        let server_keys_dir = self.get_server_keys_dir();
        if !server_keys_dir.exists() {
            println_step(&format!("Creating keys directory: {}", server_keys_dir.display()), 2);
            fs::create_dir_all(&server_keys_dir)
                .context("Failed to create server keys directory")?;
        }

        // Get workshop content directory and convert to absolute path
        let workshop_dir_raw = self.get_workshop_content_dir();
        let workshop_dir = std::path::absolute(workshop_dir_raw)
            .context("Failed to convert workshop directory to absolute path")?;

        for mod_entry in &self.config.mods.mod_list {
            self.install_single_mod(mod_entry.id, &mod_entry.name, &workshop_dir, &server_keys_dir)?;
        }

        println!();
        println_success("Mod installation completed", 0);
        Ok(())
    }

    /// Install a single mod
    fn install_single_mod(&self, workshop_id: u64, mod_name: &str, workshop_dir: &PathBuf, server_keys_dir: &PathBuf) -> Result<()> {
        println_step(&format!("Installing mod: {mod_name} ({workshop_id})"), 2);

        // Path to the downloaded mod in workshop - convert to absolute path
        let mod_workshop_dir = workshop_dir.join(workshop_id.to_string());
        
        if !mod_workshop_dir.exists() {
            println_failure(&format!("Mod directory not found: {}", mod_workshop_dir.display()), 3);
            println_step("Make sure the mod was downloaded successfully via SteamCMD", 3);
            return Err(anyhow!("Mod {} ({}) not found in workshop directory", mod_name, workshop_id));
        }

        // Create symlink for the mod directory in server install dir
        let server_mod_dir = self.server_install_dir.join(format!("@{mod_name}"));
        self.create_mod_symlink(&mod_workshop_dir, &server_mod_dir, mod_name)?;

        // Copy mod keys if they exist
        let mod_keys_dir = server_mod_dir.join("keys");
        if mod_keys_dir.exists() {
            self.copy_mod_keys(&mod_keys_dir, server_keys_dir, mod_name)?;
        } else {
            println_step(&format!("No keys directory found for mod: {mod_name}"), 3);
        }

        Ok(())
    }

    /// Create a symbolic link for a mod directory
    fn create_mod_symlink(&self, source: &PathBuf, target: &PathBuf, mod_name: &str) -> Result<()> {
        // Debug: Print the paths we're working with
        println_step(&format!("Source path: {}", source.display()), 3);
        println_step(&format!("Target path: {}", target.display()), 3);
        
        // Verify source exists
        if !source.exists() {
            return Err(anyhow!("Source mod directory does not exist: {}", source.display()));
        }

        // Check if target already exists
        if target.exists() {
            println_step(&format!("Symlink already exists: {}", target.display()), 3);
            return Ok(());
        }

        // Create the symbolic link
        // symlink_dir(original, link) - creates 'link' pointing to 'original'
        symlink_dir(source, target)
            .context(format!("Failed to create symlink from {} to {}", source.display(), target.display()))?;
        
        println_step(&format!("Created symlink for mod: {mod_name}"), 3);

        Ok(())
    }

    /// Copy mod keys to server keys directory
    fn copy_mod_keys(&self, mod_keys_dir: &PathBuf, server_keys_dir: &PathBuf, mod_name: &str) -> Result<()> {
        println_step(&format!("Copying keys for mod: {mod_name}"), 3);

        // Get all .bikey files from the mod's keys directory
        let entries = fs::read_dir(mod_keys_dir)
            .context("Failed to read mod keys directory")?;

        let mut keys_copied = 0;
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Only copy .bikey files
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension.to_string_lossy().to_lowercase() == "bikey" {
                        let file_name = path.file_name()
                            .ok_or_else(|| anyhow!("Invalid file name"))?;
                        let target_path = server_keys_dir.join(file_name);
                        
                        fs::copy(&path, &target_path)
                            .context(format!("Failed to copy key file: {file_name:?}"))?;
                        keys_copied += 1;
                        println_step(&format!("Copied key: {file_name:?}"), 4);
                    }
                }
            }
        }

        if keys_copied > 0 {
            println_step(&format!("Copied {keys_copied} key(s) for mod: {mod_name}"), 3);
        } else {
            println_step(&format!("No new keys to copy for mod: {mod_name}"), 3);
        }

        Ok(())
    }

    /// Copy mod keys to server keys directory
    fn _create_mod_keys_symlink(&self, mod_keys_dir: &PathBuf, server_keys_dir: &PathBuf, mod_name: &str) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    /// Get the workshop content directory path
    fn get_workshop_content_dir(&self) -> PathBuf {
        self.steamcmd_dir
            .join("steamapps")
            .join("workshop")
            .join("content")
            .join(self.config.server.game_app_id.to_string())
    }

    fn get_server_keys_dir(&self) -> PathBuf {
        self.server_install_dir
            .join("keys")
    }

    /// Clean up old/unused mod symlinks that are no longer in the config
    pub fn cleanup_unused_mods(&self) -> Result<()> {
        println_step("Cleaning up unused mod symlinks...", 1);

        // Get all @ModName directories in server install directory
        let entries = fs::read_dir(&self.server_install_dir)
            .context("Failed to read server install directory")?;

        let configured_mod_names: Vec<String> = self.config.mods.mod_list
            .iter()
            .map(|mod_entry| format!("@{}", mod_entry.name))
            .collect();

        let mut cleaned_count = 0;
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Check if it's a mod directory (starts with @)
                    if dir_name.starts_with('@') {
                        // Check if this mod is still in our configuration
                        if !configured_mod_names.contains(&dir_name.to_string()) {
                            println_step(&format!("Removing unused mod: {dir_name}"), 2);
                            
                            // Check if it's a symlink or real directory
                            if path.read_link().is_ok() {
                                // It's a symlink, just remove it
                                fs::remove_file(&path)
                                    .context(format!("Failed to remove mod symlink: {dir_name}"))?;
                            } else {
                                // It's a real directory, remove it recursively
                                fs::remove_dir_all(&path)
                                    .context(format!("Failed to remove mod directory: {dir_name}"))?;
                            }
                            cleaned_count += 1;
                        }
                    }
                }
            }
        }

        if cleaned_count > 0 {
            println_success(&format!("Cleaned up {cleaned_count} unused mod(s)"), 1);
        } else {
            println_success("No unused mods to clean up", 1);
        }

        Ok(())
    }
}
