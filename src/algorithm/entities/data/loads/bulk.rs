//! Промежуточные структуры для serde_json для парсинга данных груза
use serde::{Deserialize, Serialize};
use crate::algorithm::entities::data::DataArray;
use super::{AssignmentType, CargoType};
///
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadBulkData {
    /// ID груза
    pub space_id: usize,
    /// масса, т
    pub mass: Option<f64>,
    /// Общая масса, т
    pub volume: Option<f64>,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// Тип груза судна
    pub cargo_type: CargoType,
    /// Груз - лес и может намокать и обмерзать
    pub timber: bool,
    /// Груз на палубе, имеет площадь поверхностей
    pub is_on_deck: bool,
    /// Груз - контейнер
    pub container: Option<bool>,
    /// Диапазон по длинне, м
    pub bound_x1: f64,
    pub bound_x2: f64,
    /// Диапазон по ширине
    pub bound_y1: Option<f64>,
    pub bound_y2: Option<f64>,
    /// Диапазон по высоте
    pub bound_z1: Option<f64>,
    pub bound_z2: Option<f64>,
    /// Отстояние центра величины, м
    pub mass_shift_x: Option<f64>,
    pub mass_shift_y: Option<f64>,
    pub mass_shift_z: Option<f64>,
    /// Площадь горизонтальной поверхности, м^2
    pub horizontal_area: Option<f64>,
    /// Площадь вертикальной поверхности, м^2
    pub vertical_area: Option<f64>,
    /// Смещение центра площади вертикальной поверхности, м
    pub vertical_area_shift_x: Option<f64>,
    pub vertical_area_shift_y: Option<f64>,
    pub vertical_area_shift_z: Option<f64>,
}
//
impl std::fmt::Display for LoadBulkData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoadBulkData(name:{} mass:{} general_category:{} timber:{} is_on_deck:{} container:{} bound_x:({}, {}) bound_y:({}, {}) bound_z:({}, {}) 
            mass_shift:({}, {}, {}) horizontal_area:{} vertical_area:{} vertical_area_shift_y:({}, {}, {}) )",
            self.name,
            self.mass.unwrap_or(0.),            
            self.general_category,
            self.timber,
            self.is_on_deck,
            self.container.unwrap_or(false),
            self.bound_x1,
            self.bound_x2,
            self.bound_y1.unwrap_or(0.),
            self.bound_y2.unwrap_or(0.),
            self.bound_z1.unwrap_or(0.),
            self.bound_z2.unwrap_or(0.),
            self.mass_shift_x.unwrap_or(0.),
            self.mass_shift_y.unwrap_or(0.),
            self.mass_shift_z.unwrap_or(0.),
            self.horizontal_area.unwrap_or(0.),
            self.vertical_area.unwrap_or(0.),
            self.vertical_area_shift_x.unwrap_or(0.),
            self.vertical_area_shift_y.unwrap_or(0.),
            self.vertical_area_shift_z.unwrap_or(0.),
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
