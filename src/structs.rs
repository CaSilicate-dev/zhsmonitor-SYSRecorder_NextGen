use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub interval: u32,
    pub min_connections: u32,
    pub max_connections: u32,
    pub network_card_name: String,
    pub db_url: String,
    pub db_table: String,
}
