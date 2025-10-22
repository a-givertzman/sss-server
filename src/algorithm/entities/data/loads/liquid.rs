//! Промежуточные структуры для serde_json для парсинга данных груза
use std::collections::HashMap;

use super::{AssignmentType, LiquidCargoType};
use crate::algorithm::entities::{Position, data::DataArray, ship_model::LiquidData};
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
    pub fn data(&self) -> Option<LiquidData> {
        if self.mass <= 0. {
            return None;
        }
        let volume = if let Some(volume) = self.volume {
            volume
        } else {
            if let Some(density) = self.density && density > 0. {
                self.mass / density
            } else {
                return None;
            }
        };
        Some(LiquidData {
            assigned_id:  self.assigned_id,
       //     cargo_id: self.cargo_id,
            space_id: self.space_id.clone(),
            mass: self.mass,
            volume,
        })
    }
}
/// Массив данных по грузам
pub type LoadLiquidArray = DataArray<LoadLiquidData>;
//
impl LoadLiquidArray {
    pub fn data(self) -> HashMap<usize, LoadLiquidData> {
        self.data.into_iter().filter(|v| v.mass > 0.).map(|v| (v.assigned_id, v)).collect()
    }
}
