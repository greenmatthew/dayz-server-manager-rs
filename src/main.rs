use anyhow::{Result};

mod ui;
use ui::banner::print_banner;

mod lock;
use lock::check_if_initialized;

mod config;
use config::Config;


const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    print_banner();

    if !check_if_initialized()? {
        println!("\nInstallation aborted.");
        return Ok(());
    }

    // Check and load configuration
    let config = Config::check_and_load()?;

    println!("Config:");
    println!("  Server:");
    println!("    steamcmd_dir: {}", config.server.steamcmd_dir);
    println!("    server_app_id: {}", config.server.server_app_id);
    println!("    game_app_id: {}", config.server.game_app_id);
    println!("    username: {}", config.server.username);
    println!("  Mods:");
    if config.mods.mod_list.is_empty() {
        println!("    (no mods configured)");
    } else {
        for (index, mod_entry) in config.mods.mod_list.iter().enumerate() {
            println!("    {}. {}", index + 1, mod_entry);
        }
    }
    
    Ok(())
}