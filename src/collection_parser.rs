use anyhow::{Result, anyhow};
use scraper::{Html, Selector};
use crate::config::mod_entry::ModEntry;

pub struct SteamCollectionParser;

impl SteamCollectionParser {
    /// Parse a Steam Workshop collection HTML page and extract mod entries
    pub fn parse_collection_html(html_content: &str) -> Result<Vec<ModEntry>> {
        let document = Html::parse_document(html_content);
        
        // CSS selector to find all links containing workshop filedetails
        let selector = Selector::parse("a[href*='/sharedfiles/filedetails/?id=']")
            .map_err(|e| anyhow!("Failed to create CSS selector: {:?}", e))?;
        
        let mut mods = Vec::new();
        
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                // Extract mod ID from URL like: https://steamcommunity.com/sharedfiles/filedetails/?id=1559212036
                if let Some(id_str) = Self::extract_mod_id_from_url(href) {
                    if let Ok(id) = id_str.parse::<u64>() {
                        // Look for the workshop title within this link
                        let title_selector = Selector::parse(".workshopItemTitle").unwrap();
                        if let Some(title_element) = element.select(&title_selector).next() {
                            let name = title_element.text().collect::<String>().trim().to_string();
                            
                            if !name.is_empty() {
                                mods.push(ModEntry { id, name });
                            }
                        }
                    }
                }
            }
        }
        
        if mods.is_empty() {
            return Err(anyhow!("No workshop items found in the HTML. This might not be a valid Steam Workshop collection page."));
        }
        
        Ok(mods)
    }
    
    /// Extract mod ID from Steam Workshop URL
    fn extract_mod_id_from_url(url: &str) -> Option<&str> {
        url.find("?id=").map(|id_start| {
            let id_part = &url[id_start + 4..]; // Skip "?id="
            
            // Find end of ID (either end of string or next parameter)
            let id_end = id_part.find('&').unwrap_or(id_part.len());
            &id_part[..id_end]
        })
    }
    
    /// Verify if the HTML appears to be a Steam Workshop collection page
    pub fn is_collection_page(html_content: &str) -> bool {
        let document = Html::parse_document(html_content);
        
        // Look for collection-specific elements
        let collection_selector = Selector::parse(".collectionChildren").unwrap();
        let workshop_selector = Selector::parse(".workshopItem").unwrap();
        
        document.select(&collection_selector).next().is_some() && 
        document.select(&workshop_selector).next().is_some()
    }
    
    /// Get collection title from HTML
    pub fn get_collection_title(html_content: &str) -> Option<String> {
        let document = Html::parse_document(html_content);
        
        // Try to find the collection title
        let title_selector = Selector::parse(".workshopItemTitle").unwrap();
        
        if let Some(title_element) = document.select(&title_selector).next() {
            let title = title_element.text().collect::<String>().trim().to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }
        
        // Fallback to page title
        let page_title_selector = Selector::parse("title").unwrap();
        if let Some(title_element) = document.select(&page_title_selector).next() {
            let full_title = title_element.text().collect::<String>();
            // Steam titles are usually like "Steam Workshop::Collection Name"
            if let Some(title_part) = full_title.split("::").nth(1) {
                return Some(title_part.trim().to_string());
            }
        }
        
        None
    }
}
