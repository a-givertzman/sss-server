//! Промежуточные структуры для serde_json для парсинга данных зерновых перегородок
use super::UnitCargoType;
use crate::algorithm::entities::data::{DataArray, loads::{AssignmentType, LoadUnitData}};
use serde::Deserialize;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BulkheadData {
    /// Имя перегородки
    pub name: String,    
    /// Помещение в котором находится перегородка
    pub code: String,
    /// масса, т
    pub mass: f64,
    /// Границы груза в связанной с судном системой координат
    pub bound_x1: f64,
    pub bound_x2: f64,
    pub mass_shift_x: f64,
    pub mass_shift_y: f64,
    pub mass_shift_z: f64,
}
/// Массив данных по контейнерам
pub type BulkheadDataArray = DataArray<BulkheadData>;
// Преобразуется в стандартный груз
impl BulkheadDataArray {
    pub fn data(self) -> Vec<LoadUnitData> {
        self.data.into_iter().filter(|v| v.mass > 0.).map(|v| 
        LoadUnitData{
            cargo_id: 0,
            cargo_name: v.name,
            code: v.code.clone(),
            space_name: v.code.clone(),
            assignment_id: 0,
            assigment_type: AssignmentType::Unspecified,
            cargo_type: UnitCargoType::GrainBulkhead,
            mass: v.mass,
            mass_shift_x: Some(v.mass_shift_x),
            mass_shift_y: Some(v.mass_shift_y),
            mass_shift_z: Some(v.mass_shift_z),
            stowage_factor: None,
            permeability: None,
            volume: None,
            icing_area: None,
            centre_of_icing_area_x: None,
            centre_of_icing_area_y: None,
            centre_of_icing_area_z: None,
            windage_area: None,
            centre_of_windage_area_x: None,
            centre_of_windage_area_y: None,
            centre_of_windage_area_z: None,
            bound_x1: Some(v.bound_x1),
            bound_x2: Some(v.bound_x2),
            bound_y1: None,
            bound_y2: None,
            bound_z1: None,
            bound_z2: None,
        } ).collect()
    }
}
