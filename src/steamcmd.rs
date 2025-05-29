use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;
use std::io::Cursor;
use curl::easy::Easy;
use std::process::{Command, Stdio};

use crate::ui::status::{println_failure, println_step, println_success};
use crate::ui::prompt::prompt_yes_no;

const STEAMCMD_EXE: &str = "steamcmd.exe";
const STEAMCMD_DOWNLOAD_URL: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip";

pub struct SteamCmdManager {
    steamcmd_dir: PathBuf,
}

impl SteamCmdManager {
    /// Create a new ``SteamCmdManager`` and ensure steamcmd is installed
    pub fn new(steamcmd_dir: &str) -> Result<Self> {
        let steamcmd_dir_path = PathBuf::from(steamcmd_dir);
        let manager = Self {
            steamcmd_dir: steamcmd_dir_path,
        };
        
        // Check and install steamcmd during construction
        manager.check_and_install()?;
        Ok(manager)
    }

    /// Install or update a Steam application (like DayZ server)
    #[allow(clippy::doc_markdown)]
    pub fn install_or_update_app(
        &self, 
        install_dir: &str, 
        username: &str, 
        app_id: u32, 
        validate: bool
    ) -> Result<()> {
        let mut args = vec![
            "+force_install_dir".to_string(),
            install_dir.to_string(),
            "+login".to_string(),
            username.to_string(),
            "+app_update".to_string(),
            app_id.to_string(),
        ];
        
        if validate {
            args.push("validate".to_string());
        }
        
        args.push("+quit".to_string());
        
        self.run_steamcmd_with_args(&args)
    }

    /// Install or update a Steam Workshop mod
    pub fn download_or_update_mod(
        &self, 
        username: &str, 
        app_id: u32, 
        workshop_id: u64, 
        validate: bool
    ) -> Result<PathBuf> {
        let mut args = vec![
            "+login".to_string(),
            username.to_string(),
            "+workshop_download_item".to_string(),
            app_id.to_string(),
            workshop_id.to_string(),
        ];
        
        if validate {
            args.push("validate".to_string());
        }
        
        args.push("+quit".to_string());
        
        self.run_steamcmd_with_args(&args)?;

        let mut mod_path = self.get_workshop_content_dir(app_id)
            .join(workshop_id.to_string());
        mod_path = std::path::absolute(mod_path)
            .context("Failed to convert workshop directory to absolute path")?;

        // Return the path where steamcmd cached the mod
        Ok(mod_path)
    }

    /// Get the path to the steamcmd executable
    pub fn get_exe_path(&self) -> PathBuf {
        self.steamcmd_dir.join(STEAMCMD_EXE)
    }

    /// Get workshop content directory for a specific game
    pub fn get_workshop_content_dir(&self, game_app_id: u32) -> PathBuf {
        self.steamcmd_dir
            .join("steamapps")
            .join("workshop")
            .join("content")
            .join(game_app_id.to_string())
    }

    /// Check if steamcmd is installed and handle installation if needed
    fn check_and_install(&self) -> Result<()> {
        let steamcmd_exe_path = self.get_exe_path();

        // Check if steamcmd.exe exists
        if steamcmd_exe_path.exists() {
            println_success("SteamCMD found", 0);
            return Ok(());
        }

        println_failure("SteamCMD missing", 0);

        // Check if directory exists
        if !self.steamcmd_dir.exists() {
            println_step(&format!("Creating SteamCMD directory: {}", self.steamcmd_dir.display()), 1);
            fs::create_dir_all(&self.steamcmd_dir)
                .context("Failed to create SteamCMD directory")?;
        }

        // Check if directory is empty (if it existed)
        if !self.is_directory_empty()? {
            return Err(anyhow!(
                "SteamCMD directory is not empty: '{}'\nPlease clear the directory or choose a different path in config.toml",
                self.steamcmd_dir.display()
            ));
        }

        // Ask user if they want to install SteamCMD
        println_step(&format!("Would you like to install SteamCMD at: \"{}\"", self.steamcmd_dir.display()), 1);
        
        if !prompt_yes_no("Proceed with installation?", true, 1)? {
            return Err(anyhow!("SteamCMD installation declined by user"));
        }

        self.download_and_install()?;
        println_success("SteamCMD installed successfully", 0);
        
        Ok(())
    }

    fn download_and_install(&self) -> Result<()> {
        println_step("Downloading SteamCMD...", 2);
        
        // Download the zip file
        let zip_data = Self::download_steamcmd_zip()?;
        
        println_step("Extracting SteamCMD...", 2);
        
        // Extract the zip file
        self.extract_zip(zip_data)?;
        
        println_success("SteamCMD extraction complete", 2);
        
        Ok(())
    }

    /// Run SteamCMD with arguments, allowing interactive input
    #[allow(clippy::doc_markdown)]
    fn run_steamcmd_with_args(&self, args: &[String]) -> Result<()> {
        let steamcmd_exe = self.get_exe_path();
        
        println!("Running SteamCMD with args: {args:?}");
        
        // Use spawn() instead of output() to allow interactive input
        let mut child = Command::new(&steamcmd_exe)
            .args(args)
            .stdin(Stdio::inherit())   // Allow user input
            .stdout(Stdio::inherit())  // Show output directly
            .stderr(Stdio::inherit())  // Show errors directly
            .spawn()
            .context("Failed to execute SteamCMD")?;
        
        // Wait for the process to complete
        let status = child.wait()
            .context("Failed to wait for SteamCMD process")?;
        
        if !status.success() {
            return Err(anyhow!(
                "SteamCMD failed with exit code: {:?}", 
                status.code()
            ));
        }

        Ok(())
    }

    /// Check if the steamcmd directory is empty
    fn is_directory_empty(&self) -> Result<bool> {
        let entries = fs::read_dir(&self.steamcmd_dir)
            .context("Failed to read SteamCMD directory")?;
        
        Ok(entries.count() == 0)
    }

    /// Download steamcmd zip file using curl
    fn download_steamcmd_zip() -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut handle = Easy::new();
        
        handle.url(STEAMCMD_DOWNLOAD_URL)?;
        handle.follow_location(true)?;
        handle.timeout(std::time::Duration::from_secs(60))?; // 60 seconds total timeout
        
        {
            let mut transfer = handle.transfer();
            transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;
            transfer.perform()?;
        }
        
        // Check HTTP status
        let response_code = handle.response_code()?;
        if response_code != 200 {
            return Err(anyhow!("HTTP error {}: Failed to download SteamCMD", response_code));
        }
        
        if data.is_empty() {
            return Err(anyhow!("Downloaded file is empty"));
        }
        
        println_success(&format!("Downloaded {} bytes", data.len()), 3);
        Ok(data)
    }

    /// Extract zip file to steamcmd directory
    fn extract_zip(&self, zip_data: Vec<u8>) -> Result<()> {
        use zip::ZipArchive;
        use std::io::Read;
        
        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)
            .context("Failed to read zip archive")?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to access file in zip")?;
            
            let file_path = self.steamcmd_dir.join(file.name());
            
            // Create parent directories if needed
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create parent directories")?;
            }
            
            // Extract file
            if file.is_dir() {
                fs::create_dir_all(&file_path)
                    .context("Failed to create directory")?;
            } else {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)
                    .context("Failed to read file from zip")?;
                
                fs::write(&file_path, contents)
                    .context("Failed to write extracted file")?;
                
                println_step(&format!("Extracted: {}", file.name()), 3);
            }
        }
        
        Ok(())
    }
}