//! Промежуточные структуры для serde_json для парсинга данных
//! Координаты точки на корпусе судна
//! относительно центра корпуса судна
use serde::{Deserialize, Serialize};
/// Координаты точки на корпусе судна
/// относительно центра
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PointData {
    /// id критерия
    pub criterion_id: i32,
    /// Имя
    pub name: String,
    /// Координаты относительно центра корпуса судна, м
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
//
impl PointData {
    //
    pub fn new(criterion_id: i32, name: String, x: f64, y: f64, z: f64) -> Self {
        Self {
            criterion_id,
            name,
            x,
            y,
            z,
        }
    }
}
//
impl std::fmt::Display for PointData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PointData(criterion_id:{} name:{} pos:(x:{} y:{} z:{}))",
            self.criterion_id, self.name, self.x, self.y, self.z
        )
    }
}
