use clap::Parser;

#[derive(Parser)]
#[command(
    name = "dzsm",
    version = env!("CARGO_PKG_VERSION"),
    about = "DayZ Server Manager - Download, update, and run DayZ servers with mod support"
)]
pub struct CliArgs {
    // No additional arguments yet - just help (-h/--help) is automatically provided by clap
}

impl CliArgs {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}