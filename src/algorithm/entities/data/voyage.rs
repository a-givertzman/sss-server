//! Промежуточные структуры для serde_json для парсинга данных судна
use super::DataArray;
use serde::{Deserialize, Serialize};
/// Общие по судну и расчету
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub struct Voyage {
    /// плотность воды
    pub density: f64,
    /// Эксплуатационная скорость судна, m/s
    pub operational_speed: f64,
    /// Тип облединения корпуса судна
    pub icing_type: String,
    /// Тип облединения палубного груза - леса
    pub icing_timber_type: String,
    /// Тип акватории
    pub area: Option<String>,
    /// Курс судна в северо-восточной системе координат φ
    pub course_angle: f64,
    /// Курсовой угол волнения в северо-восточной системе координат β
    pub wave_heading_angle: f64,
    /// длину волны λ, m
    pub wave_length: f64,
    /// Текущая скорость корабля, m/s
    pub current_speed: f64,
}
//
pub type VoyageArray = DataArray<Voyage>;
//
impl std::fmt::Display for Voyage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Voyage(density:{}, operational_speed:{}, icing_type:{}, icing_timber_type:{} area:{} course_angle:{} wave_heading_angle:{} wave_length:{} current_speed:{})",
            self.density,
            self.operational_speed,
            self.icing_type,
            self.icing_timber_type,
            self.area.as_ref().unwrap_or(&"-".to_owned()),
            self.course_angle,
            self.wave_heading_angle,
            self.wave_length,
            self.current_speed,
        )
    }
}
