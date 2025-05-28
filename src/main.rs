use anyhow::{Context, Result};

mod ui;
use ui::banner::print_banner;

mod lock;
use lock::check_if_initialized;

mod config;
use config::Config;

mod steamcmd;
use steamcmd::SteamCmdManager;

mod mods;
use mods::ModsManager;

mod server;
use server::ServerManager;

mod collection_parser;

mod collection_fetcher;
use collection_fetcher::CollectionFetcher;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    print_banner();

    // Get current working directory for server installation
    let server_install_dir = std::env::current_dir()?
        .to_string_lossy()
        .to_string();

    if !check_if_initialized()? {
        println!("\nInstallation aborted.");
        return Ok(());
    }

    // Check and load configuration - exits gracefully if config needs editing
    let config = Config::check_and_load(&server_install_dir)?;

    // Initialize SteamCMD manager with config and server install directory
    let steamcmd_manager = SteamCmdManager::new(config.clone(), &server_install_dir);
    
    // Check and install SteamCMD if needed (always validates)
    steamcmd_manager.check_and_install()?;
    
    // Update server (always validates)
    steamcmd_manager.update_server()?;
    
    // Update/validate mods
    steamcmd_manager.update_mods()?;

    // Initialize mods manager and install/update mods
    let mods_manager = ModsManager::new(config.clone(), &server_install_dir);
    mods_manager.cleanup_unused_mods()?;
    mods_manager.install_mods()?;

    // Initialize and run the DayZ server
    let server_manager = ServerManager::new(config.clone(), &server_install_dir);
    server_manager.run_server()?;
    
    Ok(())
}