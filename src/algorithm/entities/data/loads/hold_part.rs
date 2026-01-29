//! Промежуточные структуры для serde_json для парсинга данных отделений трюма
use std::collections::HashMap;

use crate::algorithm::entities::data::DataArray;
use serde::Deserialize;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct HoldPartData {
    /// ID помещения из документации
    pub code: String,    
    /// Индекс группы (трюма) 
    pub group_id: usize,
    /// Индекс помещения в группе
    pub group_index: usize,
}
/// Массив данных отделений трюма
pub type HoldPartDataArray = DataArray<HoldPartData>;
//
impl HoldPartDataArray {
    pub fn data(self) -> HashMap<String, HoldPartData> {
        self.data.into_iter().map(|v| (v.code, v)).collect()
    }
}
