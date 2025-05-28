use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

mod ui;
use ui::banner::print_banner;
use ui::status::{println_failure, println_step, println_step_concat, println_success};
use ui::prompt::prompt_yes_no;

mod config;
use config::config::Config;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOCK_FILE: &str = "manager.lock";

fn check_if_initialized() -> Result<bool> {
    // Check if directory is already managed
    let lock_path = Path::new(LOCK_FILE);
    if lock_path.exists() {
        ui::status::println_success("Found existing DZSM setup", 0);
        Ok(true)
    } else {
        println_failure("No existing DZSM setup found", 0);
        initialize()
    }
}

fn initialize() -> Result<bool> {
    let cwd = std::env::current_dir().context("Failed to get current working directory")?;
    let cwd_str = cwd.display();

    println_step(&format!("Would you like to use the current working directory: \"{cwd_str}\""), 1);
    println_step_concat("This will install DZSM configuration files", 1);
    println_step_concat("along with DayZ server and mod files to this directory.", 1);

    if !prompt_yes_no("Proceed with installation?", false, 1)? {
        return Ok(false);
    }

    println_step("Initializing DZSM in current directory...", 1);
    
    // Create lock file to mark directory as managed
    println_step("Creating manager.lock file", 2);
    match fs::write(LOCK_FILE, format!("Managed by DZSM v{VERSION} - DayZ Server Manager\n")) {
        Ok(()) => {
            println_success("Created new DZSM setup", 0);
            Ok(true)
        }
        Err(e) => {
            // print_status(&format!("Failed to create manager.lock: {}", e), 0);
            Err(e).context("Failed to create 'manager.lock' file")
        }
    }
}

fn main() -> Result<()> {
    print_banner();

    if !check_if_initialized()? {
        println!("\nInstallation aborted.");
        return Ok(());
    }
    
    Ok(())
}