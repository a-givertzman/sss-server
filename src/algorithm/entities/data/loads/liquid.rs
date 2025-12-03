//! Промежуточные структуры для serde_json для парсинга данных груза
use std::collections::HashMap;

use super::{AssignmentType, LiquidCargoType};
use crate::algorithm::entities::{data::{DataArray, loads::CompartmentPurpose}, ship_model::LiquidData};
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
    pub assignment_id: usize,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// Тип жидкого груза
    pub cargo_type: LiquidCargoType,
    /// Тип груза для отсека
    pub compartment_purpose: CompartmentPurpose,
    /// масса, т
    pub mass: f64,
    /// Плотность
    pub density: Option<f64>,
    /// Обьем, м^3
    pub volume: Option<f64>,
   /// Центр отсека, размещающего груз, м
    pub mass_shift_x: Option<f64>,
    pub mass_shift_y: Option<f64>,
    pub mass_shift_z: Option<f64>,
   /// Признак использования максимального значения момента свободной поверхности жидкости
    pub use_moment_of_inertia_max: bool,  
    /// Момент свободной поверхности жидкости
    pub long_moment_of_inertia_max: Option<f64>,
    pub trans_moment_of_inertia_max: Option<f64>,
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
        let density = if let Some(density) = self.density {
            density
        } else {
            if self.mass > 0. {
                volume / self.mass
            } else {
                return None;
            }
        };
        Some(LiquidData {
            assignment_id:  self.assignment_id,
            assigment_type: self.assigment_type,
            cargo_type: self.cargo_type,
       //     cargo_id: self.cargo_id,
            space_id: self.space_id.clone(),
            use_max_moment: self.use_moment_of_inertia_max,
            is_cargo_tank: self.compartment_purpose == CompartmentPurpose::CargoTank,
            mass: self.mass,
            volume,
            density,
        })
    }
}
/// Массив данных по грузам
pub type LoadLiquidArray = DataArray<LoadLiquidData>;
//
impl LoadLiquidArray {
    pub fn data(self) -> HashMap<usize, LoadLiquidData> {
        self.data.into_iter().filter(|v| v.mass > 0.).map(|v| (v.assignment_id, v)).collect()
    }
}
