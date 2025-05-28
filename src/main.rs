use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

mod config;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_banner() {
    let banner = include_str!("../banner.ascii");

    let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);

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

fn main() -> Result<()> {
    print_banner();

    println!("Using DZSM to manage a DayZ server.\n");

    let cwd = std::env::current_dir().context("Failed to get current working directory")?;
    let cwd_str = cwd.display();

    let lock_path = Path::new("manager.lock");

    if lock_path.exists() {
        println!("Found existing DZSM-managed directory:\n  \"{cwd_str}\"\n");
    } else {
        println!("No existing DZSM setup found in:\n  \"{cwd_str}\"");
        println!("This will install DZSM configuration files");
        println!("along with DayZ server and mod files to this directory.\n");

        print!("Proceed with installation? (y/N): ");
        io::stdout().flush()?; // Ensure prompt is shown

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("\nInstallation aborted.");
            return Ok(());
        }

        println!("\nCreating 'manager.lock' to mark this directory as DZSM-managed...");

        fs::write(lock_path, format!("Managed by dayz-server-manager v{VERSION}\n"))
            .context("Failed to create 'manager.lock' file")?;
    }

    let config = config::config::Config::load_or_create("config.toml")?;
    config.save("config.toml")?;

    println!("Server install dir: {}", config.server.install_dir);
    
    Ok(())
}