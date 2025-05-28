use serde::{Deserialize, Serialize};
use crate::config::mod_entry::ModEntry;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModsConfig {
    pub mod_list: Vec<ModEntry>
}