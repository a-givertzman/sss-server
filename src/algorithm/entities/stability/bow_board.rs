//! Промежуточные структуры для serde_json для парсинга данных
//! Высота борта на носовом перпендикуляре
//! относительно центра корпуса судна
use crate::algorithm::entities::math::Position;

use super::{DataArray, PointData};
/// Высота борта на носовом перпендикуляре судна относительно центра
pub type BowBoardDataArray = DataArray<PointData>;
//
impl BowBoardDataArray {
    /// Преобразование данных в массив
    pub fn bow_board_data(&self) -> Vec<BowBoardParsedData> {
        self.data
            .iter()
            .map(|v| BowBoardParsedData {
                criterion_id: v.criterion_id,
                pos: Position::new(v.x, v.y, v.z),
            })
            .collect()
    }
}
//
#[derive(Debug, Clone, PartialEq)]
pub struct BowBoardParsedData {
    /// id
    pub criterion_id: i32,
    /// Координаты центра винта относительно центра корпуса судна, м
    pub pos: Position,
}
//
impl std::fmt::Display for BowBoardParsedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BowBoardParsedData(criterion_id:{} pos:{})", self.criterion_id, self.pos)
    }
}
