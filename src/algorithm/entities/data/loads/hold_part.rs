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
    pub fn codes(&self, group_id: usize) -> HashMap<usize, String> {
        self.data.iter().filter(|v| v.group_id == group_id).map(|v| (v.group_index, v)).collect()
    }
}
