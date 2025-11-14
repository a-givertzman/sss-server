//! Промежуточные структуры для serde_json для парсинга данных максимального объема для отсеков
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::algorithm::entities::data::DataArray;
/// Максимальный объем для отсеков
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VolumeData {
    /// ID помещения
    pub space_id: String,
    /// Максимальный объем
    pub volume_max: f64,
}
//
impl std::fmt::Display for VolumeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GrainMomentData(space_id:{}, volume_max:{} )",
            self.space_id, self.volume_max,
        )
    }
}
pub type VolumeDataArray = DataArray<VolumeData>;
//
impl VolumeDataArray {
    /// Преобразование и возвращает данные в виде мапы
    pub fn data(self) -> HashMap<String, f64> {
        self.data.into_iter().map(|v| (v.space_id, v.volume_max)).collect()
    }
}
