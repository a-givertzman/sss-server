//! Промежуточные структуры для serde_json для парсинга данных груза
use api_tools::error::str_err::StrErr;
use serde::{Deserialize, Serialize};
use crate::algorithm::entities::{data::DataArray, Bound};
/// Груз, приходящийся на шпацию
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadConstantData {
    /// Масса на шпацию
    pub mass: f64,
    /// Диапазон по длинне, м
    pub bound_x1: f64,
    pub bound_x2: f64,
}
//
impl LoadConstantData {
    //
    pub fn mass(&self, bound_x: &Bound) -> Result<f64, StrErr> {
        Ok(self.mass*Bound::new(self.bound_x1, self.bound_x2)?.part_ratio(bound_x)?)
    }
}
//
impl std::fmt::Display for LoadConstantData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoadConstantData(mass:{}, bound_x1:{}, bound_x2:{})",
            self.mass, self.bound_x1, self.bound_x2
        )
    }
}
/// Массив данных по грузам
pub type LoadConstantArray = DataArray<LoadConstantData>;
//
impl LoadConstantArray {
    pub fn data(self) -> Vec<LoadConstantData> {
        self.data
    }
}
