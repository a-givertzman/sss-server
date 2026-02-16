//! Промежуточные структуры для serde_json для парсинга данных отделений трюма образованных зерновыми перегородками
use crate::algorithm::entities::data::{DataArray, loads::HoldPartDataArray};
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
    pub group_end_index: usize,
}
/// Массив данных отделений трюма
pub type HoldCompartmentArray = DataArray<HoldCompartmentData>;
//
impl HoldCompartmentArray {
    pub fn data(self, hold_part_data: HoldPartDataArray) -> Vec<(String, Vec<String>)> {
        self.data
            .into_iter()
            .map(|v| {
                let hold_part_codes = hold_part_data.codes(v.group_id);
                let hold_part_codes = (v.group_start_index..=v.group_end_index)
                    .filter_map(|index| hold_part_codes.get(&index))
                    .map(|code| code.to_owned())
                    .collect();
                (v.code, hold_part_codes)
            })
            .collect()
    }
}
