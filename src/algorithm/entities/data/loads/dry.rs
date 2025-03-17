//! Промежуточные структуры для serde_json для парсинга данных груза
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use crate::algorithm::entities::data::DataArray;
use crate::algorithm::entities::math::Position;
//
fn deserialize_from_string<'de, D>(deserializer: D) -> Result<Position, D::Error>
where D: Deserializer<'de> {
    let buf = String::deserialize(deserializer)?;
    Position::from_str(&buf)
}
//
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadDryData {
    /// ID груза
    pub cargo_id: usize,
    /// Имя груза
    pub cargo_name: String,
    /// ID assigned
    pub assigned_id: usize,
    /// масса, т
    pub mass: Option<f64>,
    /// Центр тяжести, м
    #[serde(deserialize_with = "deserialize_from_string")]
    pub mass_shift: Option<Position>,
    /// Проницаемость, %
    pub permeability: Option<f64>,
}
/*
impl std::fmt::Display for LoadDryData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoadDryData(space_id:{} space_name:{} cargo_id:{} cargo_name:{} assigned_id:{} 
                assigment_type:{} cargo_type:{}, mass:{}, stowage_factor:{} volume:{} )",
            self.space_id,
            self.space_name,            
            self.cargo_id,
            self.cargo_name,
            self.assigned_id,
            self.assigment_type,
            self.cargo_type,
            self.mass.unwrap_or(0.),
            self.stowage_factor.unwrap_or(0.),
            self.volume.unwrap_or(0.),
        )
    }
}*/
/// Массив данных по грузам
pub type LoadDryArray = DataArray<LoadDryData>;
//
impl LoadDryArray {
    pub fn data(self) -> Vec<LoadDryData> {
        self.data
    }
}
