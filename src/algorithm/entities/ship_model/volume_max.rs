//! Промежуточные структуры для serde_json для парсинга данных максимального объема для отсеков
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::algorithm::entities::data::DataArray;
/// Максимальный объем для отсеков
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaxData {
    /// ID помещения
    pub code: String,
    /// Максимальный уровень
    pub level_max: f64,    
    /// Максимальный объем
    pub volume_max: f64,
}
//
impl std::fmt::Display for MaxData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MaxData(code:{}, level_max:{}, volume_max:{} )",
            self.code,  self.level_max, self.volume_max,
        )
    }
}
pub type MaxDataArray = DataArray<MaxData>;
//
impl MaxDataArray {
    /// Преобразование и возвращает данные в виде мапы
    pub fn data(self) -> HashMap<String, (f64, f64)> {
        self.data.into_iter().map(|v| (v.code, (v.level_max, v.volume_max))).collect()
    }
}
