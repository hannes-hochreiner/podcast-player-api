use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub db_connection: String,
}
