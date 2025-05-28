// src/collection_fetcher.rs
use anyhow::{Context, Result, anyhow};
use curl::easy::Easy;
use crate::collection_parser::SteamCollectionParser;
use crate::ui::status::{println_step, println_success, println_failure};
use crate::config::mod_entry::ModEntry;

pub struct CollectionFetcher;

impl CollectionFetcher {
    /// Fetch and parse a Steam Workshop collection by URL
    pub fn fetch_collection_mods(collection_url: &str) -> Result<Vec<ModEntry>> {
        println_step(&format!("Fetching collection: {}", collection_url), 1);
        
        // Validate URL format
        if !collection_url.contains("steamcommunity.com") || !collection_url.contains("filedetails") {
            return Err(anyhow!("Invalid Steam Workshop collection URL"));
        }
        
        // Download the HTML
        let html_content = Self::download_page(collection_url)?;
        
        // Verify it's a collection page
        if !SteamCollectionParser::is_collection_page(&html_content) {
            return Err(anyhow!("URL does not appear to be a Steam Workshop collection"));
        }
        
        // Get collection title for user feedback
        if let Some(title) = SteamCollectionParser::get_collection_title(&html_content) {
            println_step(&format!("Found collection: '{}'", title), 2);
        }
        
        // Parse the mods
        let mods = SteamCollectionParser::parse_collection_html(&html_content)
            .context("Failed to parse collection HTML")?;
        
        println_success(&format!("Successfully parsed {} mods from collection", mods.len()), 1);
        
        for (i, mod_entry) in mods.iter().enumerate() {
            println_step(&format!("{}. {} ({})", i + 1, mod_entry.name, mod_entry.id), 2);
        }
        
        Ok(mods)
    }
    
    /// Download HTML content from URL
    fn download_page(url: &str) -> Result<String> {
        let mut html_content = Vec::new();
        let mut handle = Easy::new();
        
        handle.url(url)?;
        handle.follow_location(true)?;
        handle.timeout(std::time::Duration::from_secs(30))?;
        
        // Set a user agent to avoid being blocked
        handle.useragent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")?;
        
        {
            let mut transfer = handle.transfer();
            transfer.write_function(|new_data| {
                html_content.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;
            transfer.perform()?;
        }
        
        let response_code = handle.response_code()?;
        if response_code != 200 {
            return Err(anyhow!("HTTP error {}: Failed to fetch collection page", response_code));
        }
        
        String::from_utf8(html_content)
            .context("Failed to decode HTML as UTF-8")
    }
}
