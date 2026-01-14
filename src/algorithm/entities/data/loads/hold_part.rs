//! Промежуточные структуры для serde_json для парсинга данных отделений трюма
use crate::algorithm::entities::data::DataArray;
use serde::Deserialize;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct HoldPartData {
    /// Индекс группы (трюма) 
    pub group_id: usize,
    /// Индекс помещения в группе
    pub group_index: usize,
    /// ID помещения
    pub space_id: String,
}
/// Массив данных отделений трюма
pub type HoldPartDataArray = DataArray<HoldPartData>;
