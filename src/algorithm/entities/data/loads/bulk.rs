//! Промежуточные структуры для serde_json для парсинга данных груза
use serde::{Deserialize, Serialize};
use crate::algorithm::entities::data::DataArray;
use super::{AssignmentType, BulkCargoType, CargoType};
///
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadBulkData {
    /// ID помещения
    pub space_id: usize,
    /// Имя помещения
    pub space_name: String,
    /// ID груза
    pub cargo_id: usize,
    /// Имя груза
    pub cargo_name: String,
    /// ID assigned
    pub assigned_id: usize,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// Тип сыпучего груза
    pub cargo_type: BulkCargoType,
    /// масса, т
    pub mass: Option<f64>,
    /// Средний удельный погрузочный объем, м^3/т
    pub stowage_factor: Option<f64>,
    /// Обьем, м^3
    pub volume: Option<f64>,
}
//
impl std::fmt::Display for LoadBulkData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoadBulkData(space_id:{} space_name:{} cargo_id:{} cargo_name:{} assigned_id:{} 
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
}
/// Массив данных по грузам
pub type LoadBulkArray = DataArray<LoadBulkData>;
//
impl LoadBulkArray {
    pub fn data(self) -> Vec<LoadBulkData> {
        self.data
    }
}
