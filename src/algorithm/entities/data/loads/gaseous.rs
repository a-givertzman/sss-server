//! Промежуточные структуры для serde_json для парсинга данных груза
use serde::Deserialize;
use crate::algorithm::entities::{data::DataArray, Position};
use super::AssignmentType;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LoadGaseousData {
    /// ID груза
    pub cargo_id: usize,
    /// Имя груза
    pub cargo_name: String,   
    /// ID помещения
    pub space_id: String,
    /// Имя помещения
    pub space_name: String,
    /// ID assigned
    pub assigned_id: usize,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,    
    /// масса, т
    pub mass: f64,
    /// Общая масса, т
    pub volume: Option<f64>,
    /// Центр отсека, размещающего газ, м
    pub mass_shift: Option<Position>,
}
//
/*impl std::fmt::Display for LoadGaseousData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoadGaseousData(name:{} mass:{} general_category:{} timber:{} is_on_deck:{} container:{} bound_x:({}, {}) bound_y:({}, {}) bound_z:({}, {}) 
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
}*/
/// Массив данных по грузам
pub type LoadGaseousArray = DataArray<LoadGaseousData>;
//
impl LoadGaseousArray {
    pub fn data(self) -> Vec<LoadGaseousData> {
        self.data
    }
}
