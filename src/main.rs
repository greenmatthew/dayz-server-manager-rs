use anyhow::{Result};

mod ui;
use ui::banner::print_banner;

mod lock;
use lock::check_if_initialized;

mod config;
use config::Config;

mod steamcmd;
use steamcmd::SteamCmdManager;

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

    // Check and load configuration
    let config = Config::check_and_load()?;

    println!("\n=== Configuration Summary ===");
    println!("Server:");
    println!("  steamcmd_dir: {}", config.server.steamcmd_dir);
    println!("  server_app_id: {}", config.server.server_app_id);
    println!("  game_app_id: {}", config.server.game_app_id);
    println!("  username: {}", config.server.username);
    println!("  install_dir: {server_install_dir}");
    
    println!("Mods:");
    if config.mods.mod_list.is_empty() {
        println!("  (no mods configured)");
    } else {
        for (index, mod_entry) in config.mods.mod_list.iter().enumerate() {
            println!("  {}. {} ({})", index + 1, mod_entry.name, mod_entry.id);
        }
    }
    println!();
    println!();

    // Initialize SteamCMD manager with config and server install directory
    let steamcmd_manager = SteamCmdManager::new(config.clone(), &server_install_dir);
    
    // Check and install SteamCMD if needed (always validates)
    steamcmd_manager.check_and_install()?;
    
    // Update server (always validates)
    steamcmd_manager.update_server()?;
    
    // Update mods (always validates)
    steamcmd_manager.update_mods()?;
    
    println!("\n=== Setup Complete ===");
    
    Ok(())
}