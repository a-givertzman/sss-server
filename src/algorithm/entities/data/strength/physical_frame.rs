//! Промежуточные структуры для serde_json для парсинга данных
//! разбиения корпуса для расчете эпюров
use serde::{Deserialize, Serialize};
use crate::algorithm::entities::data::DataArray;
/// Данные по шпангоуту
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhysicalFrameData {
    /// Индекс шпангоута
    pub frame_index: i32,
    /// Координата шпангоута по Х
    pub pos_x: f64,
}

//
impl std::fmt::Display for PhysicalFrameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PhysicalFrameData(frame_index:{}, pos_x:{})",
            self.frame_index, self.pos_x,
        )
    }
}
pub type PhysicalFrameArray = DataArray<PhysicalFrameData>;
//
impl PhysicalFrameArray {
    /// Преобразование и возвращает данные в виде вектора (индекс, координата по Х)
    pub fn data(mut self) -> Vec<(i32, f64)> {
        self
            .data
            .iter_mut()
            .map(|v| (v.frame_index, v.pos_x))
            .collect()
    }
}
