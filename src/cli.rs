use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "dzsm",
    version = env!("CARGO_PKG_VERSION"),
    about = "DayZ Server Manager - Download, update, and run DayZ servers with mod support"
)]
pub struct CliArgs {
    /// Test flag example
    #[arg(long, help = "Enable test mode")]
    pub test: bool,
    
    // Add any other command line arguments you need here
    // For example:
    // #[arg(long, help = "Skip mod validation")]
    // pub skip_validation: bool,
    
    // #[arg(long, help = "Force server update")]
    // pub force_update: bool,
}

impl CliArgs {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
