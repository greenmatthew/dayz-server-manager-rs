use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "dzsm",
    version = env!("CARGO_PKG_VERSION"),
    about = "DayZ Server Manager - Download, update, and run DayZ servers with mod support"
)]
pub struct CliArgs {
    /// Skip server validation during update
    #[arg(long = "skip-server-validation")]
    pub skip_server_validation: bool,
    
    /// Skip mod validation during update
    #[arg(long = "skip-mod-validation")]
    pub skip_mod_validation: bool,
    
    /// Skip all validation (server and mods)
    #[arg(long = "skip-validation")]
    pub skip_validation: bool,
}

impl CliArgs {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
