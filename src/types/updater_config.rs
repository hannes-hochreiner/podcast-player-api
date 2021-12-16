use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdaterConfig {
    pub db_connection: String,
}
