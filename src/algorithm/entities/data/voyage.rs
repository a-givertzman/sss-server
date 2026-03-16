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
}
//
pub type VoyageArray = DataArray<Voyage>;
//
impl std::fmt::Display for Voyage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Voyage(density:{}, operational_speed:{}, icing_type:{}, icing_timber_type:{})",
            self.density,
            self.operational_speed,
            self.icing_type,
            self.icing_timber_type,
        )
    }
}
