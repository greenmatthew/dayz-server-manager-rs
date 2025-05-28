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
    println!("\tsteamcmd_dir: {}", config.server.steamcmd_dir);
    println!("\tinstall_dir: {}", config.server.install_dir);
    
    Ok(())
}