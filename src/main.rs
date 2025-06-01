use anyhow::{Result};

mod ui;
use ui::banner::print_banner;

mod lock;
use lock::check_if_initialized;

mod config;
use config::Config;

mod steamcmd;
mod collection_parser;
mod collection_fetcher;

mod server;
use server::ServerManager;

mod cli;
use cli::CliArgs;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

fn main() -> Result<()> {
    // Parse command line arguments - this will handle -h/--help automatically
    let _args = CliArgs::parse_args();

    // Continue with normal application execution
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

    let mut server_manager = ServerManager::new(config, &server_install_dir);

    // Initialize SteamCMD
    server_manager.setup_steamcmd()?;

    // Update server (always validates)
    server_manager.install_or_update_server()?;

    // Update/validate mods
    server_manager.install_or_update_mods()?;

    // Run the DayZ server
    server_manager.run_server()?;
    
    Ok(())
}