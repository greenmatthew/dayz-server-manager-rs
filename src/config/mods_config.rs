use serde::{Deserialize, Serialize};
use crate::config::mod_entry::ModEntry;

#[derive(Debug, Deserialize, Serialize)]
pub struct ModsConfig {
    pub mod_list: Vec<ModEntry>
}