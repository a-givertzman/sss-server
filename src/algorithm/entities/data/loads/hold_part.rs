//! Промежуточные структуры для serde_json для парсинга данных отделений трюма
use crate::algorithm::entities::data::DataArray;
use serde::Deserialize;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct HoldPartData {
    /// Индекс помещения в группе
    pub group_index: usize,
    /// ID помещения
    pub space_id: usize,
    /// ID помещения слева
    pub left_space_id: usize,  
    /// ID помещения справа
    pub right_space_id: usize,
}
/// Массив данных отделений трюма
pub type HoldPartDataArray = DataArray<HoldPartData>;
