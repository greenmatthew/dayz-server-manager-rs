use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub steamcmd_dir: String,
    pub server_app_id: u32,
    pub game_app_id: u32,
    pub username: String,
}
