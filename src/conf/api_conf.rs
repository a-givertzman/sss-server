use serde::{Deserialize, Serialize};

///
/// Configuration parameters for the database connection
#[derive(Serialize, Deserialize)]
pub struct ApiConf {
    pub host: String,
    pub port: String,
    pub database: String,
}