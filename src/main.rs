use anyhow::{Result};

mod ui;
use ui::banner::print_banner;

mod lock;
use lock::check_if_initialized;

mod config;


const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    print_banner();

    if !check_if_initialized()? {
        println!("\nInstallation aborted.");
        return Ok(());
    }
    
    Ok(())
}