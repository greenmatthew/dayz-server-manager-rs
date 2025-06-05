use serde::{Deserialize, Serialize};
use crate::config::mod_entry::ModEntry;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_mod_list: Option<Vec<ModEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_collection_url: Option<String>,
}