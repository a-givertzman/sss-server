//! Промежуточные структуры для serde_json для парсинга данных контейнеров
use super::{AssignmentType, UnitCargoType};
use crate::algorithm::entities::{data::{DataArray, loads::LoadUnitData}};
use serde::Deserialize;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LoadContainerData {
    /// ID груза
    pub cargo_id: usize,
    /// ID слота
    pub slot_id: usize,
    /// Имя груза
    pub cargo_name: String,
    /// ID помещения
    pub code: String,
    /// ID assigned
    pub assignment_id: usize,    
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// масса, т
    pub mass: f64,
    /// Границы груза в связанной с судном системой координат
    pub bound_x1: Option<f64>,
    pub bound_x2: Option<f64>,
    pub bound_y1: Option<f64>,
    pub bound_y2: Option<f64>,
    pub bound_z1: Option<f64>,
    pub bound_z2: Option<f64>,
}
/// Массив данных по контейнерам
pub type LoadContainerArray = DataArray<LoadContainerData>;
// Преобразуется в стандартный груз
impl LoadContainerArray {
    pub fn data(self) -> Vec<LoadUnitData> {
        self.data.into_iter().filter(|v| v.mass > 0.).map(|v| 
        LoadUnitData{
            cargo_id: v.cargo_id,
            cargo_name: v.cargo_name,
            code: v.code,
            space_name: format!("slot_{}", v.slot_id),
            assignment_id: v.assignment_id,
            assigment_type: v.assigment_type,
            cargo_type: UnitCargoType::Container,
            mass: v.mass,
            mass_shift_x: None,
            mass_shift_y: None,
            mass_shift_z: None,
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
            bound_x1: v.bound_x1,
            bound_x2: v.bound_x2,
            bound_y1: v.bound_y1,
            bound_y2: v.bound_y2,
            bound_z1: v.bound_z1,
            bound_z2: v.bound_z2,
        } ).collect()
    }
}
