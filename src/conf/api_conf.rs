use serde::{Deserialize, Serialize};

/// Данные для инициализации api-server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiAddress {
    pub host: String,
    pub port: String,
    pub database: String,
}
/// Данные для выборки из БД
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Params {
    #[serde(alias = "ship-id")]
    pub ship_id: i32,
    #[serde(alias = "project-id")]
    pub project_id: Option<i32>,
}
/// Данные для доступа к БД
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiConf {
    #[serde(alias = "api-address")]
    pub address: ApiAddress,
    pub params: Params,
}