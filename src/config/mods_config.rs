use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ModsConfig {
    mod_list: Vec<[String; 2]>  // [WorkshopID, ModName]
}
