//! Промежуточные структуры для serde_json для парсинга данных
//! разбиения корпуса для расчете эпюров
use serde::{Deserialize, Serialize};
use crate::algorithm::entities::data::DataArray;
/// Данные по шпангоуту
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhysicalFrameData {
    /// Индекс шпангоута
    pub index: String,
    /// Координата шпангоута по Х
    pub pos_x: f64,
}

//
impl std::fmt::Display for PhysicalFrameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PhysicalFrameData(index:{}, pos_x:{})",
            self.index, self.pos_x,
        )
    }
}
pub type PhysicalFrameArray = DataArray<PhysicalFrameData>;
//
impl PhysicalFrameArray {
    /// Преобразование и возвращает данные в виде отсортированного вектора координата по Х
    pub fn data(self) -> Vec<f64> {
        let mut data: Vec<_> = self
            .data.into_iter()
            .map(|v| v.pos_x)
            .collect();
        data.sort_by(|a, b| a.partial_cmp(&b).unwrap());
        data 
    }
}
