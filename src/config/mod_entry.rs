use std::fmt;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
#[serde(try_from = "(String, String)")]
pub struct ModEntry {
    pub workshop_id: u64,
    pub name: String,
}

// Custom error type for validation
#[derive(Debug)]
pub enum ModEntryError {
    InvalidWorkshopID(String),
}

impl fmt::Display for ModEntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModEntryError::InvalidWorkshopID(s) => write!(f, "Invalid format: {s}"),
        }
    }
}

impl std::error::Error for ModEntryError {}

impl TryFrom<(String, String)> for ModEntry {
    type Error = ModEntryError;
    
    fn try_from((id, name): (String, String)) -> Result<Self, Self::Error> {
        Ok(Self {
            workshop_id: id.parse().map_err(|_| ModEntryError::InvalidWorkshopID(id))?,
            name,
        })
    }
}
