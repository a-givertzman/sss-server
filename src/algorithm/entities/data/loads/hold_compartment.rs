//! Промежуточные структуры для serde_json для парсинга данных отделений трюма образованных зерновыми перегородками
use std::collections::HashMap;

use crate::algorithm::entities::data::DataArray;
use serde::Deserialize;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]

pub struct HoldCompartmentData {
    /// ID помещения из документации
    pub code: String,        
    /// Индекс группы (трюма) 
    pub group_id: usize,
    /// Индекс первого помещения слева
    pub group_start_index: usize,
    /// Индекс последнего помещения справа    
    pub group_end_index: usize
}
/// Массив данных отделений трюма
pub type HoldCompartmentArray = DataArray<HoldCompartmentData>;
//
impl HoldCompartmentArray {
    pub fn data(self) -> HashMap<String, HoldCompartmentData> {
        self.data.into_iter().map(|v| (v.code, v)).collect()
    }
}
