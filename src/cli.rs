use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "dzsm",
    version = env!("CARGO_PKG_VERSION"),
    about = "DayZ Server Manager - Download, update, and run DayZ servers with mod support"
)]
#[allow(clippy::struct_excessive_bools)]
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

    /// Skips all SteamCMD operations,
    /// throws an error if the DayZServer64.exe is missing
    /// or if a workshop mod's source dir is missing.
    #[arg(long = "offline")]
    #[allow(clippy::doc_markdown)]
    pub offline: bool,
}

impl CliArgs {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
