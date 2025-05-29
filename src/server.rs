use anyhow::{Context, Result, anyhow};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::cell::OnceCell;

use crate::config::mod_entry::ModEntry;
use crate::steamcmd2::{self, SteamCmdManager};
use crate::ui::status::{println_step, println_success, println_failure};
use crate::config::Config;

use crate::collection_fetcher::CollectionFetcher;

const SERVER_EXE: &str = "DayZServer_x64.exe";
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

        // Fix the method call
        steamcmd.install_or_update_app(
            &self.server_install_dir.to_string_lossy(),  // Convert PathBuf to &str
            &server_config.username,        // Pass as reference
            server_config.server_app_id,    // Use server_app_id, not game_app_id
            true                           // validate parameter
        )?;  // Handle the Result with ?

        Ok(())
    }

    pub fn install_or_update_mods(&self) -> Result<()> {
        

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
        self.run_server_with_args(args)?;
        
        println_success("DayZ server has stopped", 0);
        Ok(())
    }

    /// Get individual mods from config
    fn get_individual_mods(&self) -> &[ModEntry] {
        self.config.mods.mod_list.as_deref().unwrap_or(&[])
    }

    /// Get collection mods (cached)
    fn get_collection_mods(&self) -> &[ModEntry] {
        self.collection_mod_list.get_or_init(|| {
            if let Some(collection_url) = &self.config.mods.mod_collection_url {
                if !collection_url.trim().is_empty() {
                    CollectionFetcher::fetch_collection_mods(collection_url)
                        .unwrap_or_else(|e| {
                            println_failure(&format!("Failed to fetch collection: {}", e), 0);
                            Vec::new()
                        })
                } else {
                    Vec::new()
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
        let server_config = &self.config.server;  // Take reference

        let mod_cache_path = steamcmd.download_or_update_mod(
            &server_config.username,        // Pass as reference
            server_config.server_app_id,    // Use server_app_id, not game_app_id
            workshop_id,
            true                   // validate parameter
        )?;

        println!("Source: {mod_cache_path:?}");
        println!("Target: {:?}", self.server_install_dir.join(format!("@{}", name)));

        Ok(())
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
