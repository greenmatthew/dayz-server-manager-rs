use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub steamcmd_dir: String,
    pub username: String,
}
