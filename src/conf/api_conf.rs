use serde::{Deserialize};

///
/// Данные для инициализации api-server
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ApiAddress {
    pub host: String,
    pub port: String,
    pub database: String,
}
///
/// Данные для выборки из БД
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Params {
    pub ship_id: String,
    pub project_id: String,
}
/// Данные для работы с моделью
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Model {
    pub midel_x: f64,
    pub name: String,
}
///
/// Данные для доступа к БД
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ApiConf {
    #[serde(alias = "api-address")]
    pub address: ApiAddress,
    pub params: Params,
    pub model: Model,    
}