use anyhow::{Context, Result};

mod ui;
use ui::banner::print_banner;

mod lock;
use lock::check_if_initialized;

mod config;
use config::Config;

mod steamcmd2;
use steamcmd2::SteamCmdManager;

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

    let mut server_manager = ServerManager::new(config.clone(), &server_install_dir);

    // Initialize SteamCMD
    server_manager.setup_steamcmd();

    // Initialize mods manager and install/update mods
    // let mods_manager = ModsManager::new(config.clone(), &server_install_dir);
    // mods_manager.cleanup_unused_mods()?;
    // mods_manager.install_mods()?;

    // Update server (always validates)
    server_manager.install_or_update_server();

    // Update/validate mods
    server_manager.install_or_update_mods();

    // Run the DayZ server
    server_manager.run_server();
    
    Ok(())
}