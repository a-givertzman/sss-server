//! Промежуточные структуры для serde_json для парсинга данных груза
use super::{AssignmentType, LiquidCargoType};
use crate::{algorithm::entities::{data::DataArray, Position}, ship_model::query::LiquidData};
use serde::{Deserialize, Serialize};
/// Груз без привязки к помещению, всегда твердый
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadLiquidData {
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
    /// Тип жидкого груза
    pub cargo_type: LiquidCargoType,
    /// масса, т
    pub mass: f64,
    /// Плотность
    pub density: Option<f64>,
    /// Обьем, м^3
    pub volume: Option<f64>,
    /// Центр отсека, размещающего груз, м
    pub mass_shift: Option<Position>,
    /// Признак использования максимального значения момента свободной поверхности жидкости
    pub use_moment_of_inertia_max: bool,  
    /// Момент свободной поверхности жидкости
    pub long_moment_of_inertia_max: f64,
    pub trans_moment_of_inertia_max: f64,
}
//
impl LoadLiquidData {
    pub fn data(&self) -> LiquidData {
        let volume = if let Some(volume) = self.volume {
            volume
        } else {
            if let Some(density) = self.density {
                if density > 0. {
                    self.mass / density
                } else {
                    0.
                }
            } else {
                0.
            }
        };
        LiquidData {
            cargo_id: self.cargo_id,
            space_id: self.space_id.clone(),
            mass: self.mass,
            volume,
        }
    }
}
/*
impl std::fmt::Display for LoadLiquidData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoadLiquidData(name:{} mass:{} general_category:{} timber:{} is_on_deck:{} container:{} bound_x:({}, {}) bound_y:({}, {}) bound_z:({}, {})
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
pub type LoadLiquidArray = DataArray<LoadLiquidData>;
//
impl LoadLiquidArray {
    pub fn data(self) -> Vec<LoadLiquidData> {
        self.data
    }
}
