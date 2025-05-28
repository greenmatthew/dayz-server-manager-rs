use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::ui::status::{println_failure, println_step, println_step_concat, println_success};
use crate::ui::prompt::prompt_yes_no;
use crate::VERSION;

const LOCK_FILE: &str = ".dzsm.lock";

/// Check if the current directory is already initialized with DZSM
pub fn check_if_initialized() -> Result<bool> {
    let lock_path = Path::new(LOCK_FILE);
    if lock_path.exists() {
        println_success("Found existing DZSM setup", 0);
        Ok(true)
    } else {
        println_failure("No existing DZSM setup found", 0);
        initialize()
    }
}

/// Initialize DZSM in the current directory
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
    
    create_lock_file()?;
    
    println_success("Created new DZSM setup", 0);
    Ok(true)
}

/// Create the lock file to mark directory as managed by DZSM
fn create_lock_file() -> Result<()> {
    println_step(&format!("Creating '{LOCK_FILE}' file"), 2);
    
    let lock_content = format!(
        "Managed by DZSM v{VERSION} - DayZ Server Manager\nCreated: {}\n", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    
    fs::write(LOCK_FILE, lock_content)
        .context(format!("Failed to create '{LOCK_FILE}' file"))
}
