use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

mod config;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOCK_FILE: &str = "manager.lock";

pub fn print_banner() {
    let banner = include_str!("../banner.ascii");
    let term_width = term_size::dimensions().map_or(80, |(w, _)| w);

    println!(); // Padding before banner

    for line in banner.lines() {
        let line_len = line.chars().count();
        let padding = if term_width > line_len {
            (term_width - line_len) / 2
        } else {
            0
        };
        println!("{}{}", " ".repeat(padding), line);
    }

    println!(); // Margin between banner and title

    // Center the title/version
    let title = format!("DZSM v{VERSION} - DayZ Server Manager");
    let title_len = title.chars().count();
    let padding = if term_width > title_len {
        (term_width - title_len) / 2
    } else {
        0
    };
    println!("{}{}", " ".repeat(padding), title);

    println!(); // Padding after banner
}

// Progress indicator symbols and formatting
const CHECK_MARK: &str = "✓";
const CROSS_MARK: &str = "✗";
const ARROW: &str = "→";

fn println_failure(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    println!("{indent}{CROSS_MARK} {message}");
}

fn println_step(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    println!("{indent}{ARROW} {message}");
}

fn println_step_concat(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    println!("{indent}  {message}");
}

fn print_step_concat(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    print!("{indent}  {message}");
}

fn println_success(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    println!("{indent}{CHECK_MARK} {message}");
}

fn prompt_yes_no(prompt: &str, default: bool, level: usize) -> Result<bool> {
    let options = if default { "(Y/n)" } else { "(y/N)" };
    
    println!();
    print_step_concat(&format!("{prompt} {options}: "), level);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(match input.as_str() {
        "" => default,
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => {
            println!("Please enter 'y' or 'n'");
            return prompt_yes_no(prompt, default, level);
        }
    })
}

fn check_if_initialized() -> Result<bool> {
    // Check if directory is already managed
    let lock_path = Path::new(LOCK_FILE);
    if lock_path.exists() {
        println_success("Found existing DZSM setup", 0);
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
    match fs::write(LOCK_FILE, format!("Managed by dayz-server-manager v{VERSION}\n")) {
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
    println!("Using DZSM to manage a DayZ server.\n");

    if !check_if_initialized()? {
        println!("\nInstallation aborted.");
        return Ok(());
    }
    
    Ok(())
}