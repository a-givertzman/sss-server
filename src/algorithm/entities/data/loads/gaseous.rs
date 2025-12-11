//! Промежуточные структуры для serde_json для парсинга данных груза
use std::collections::HashMap;

use sal_core::error::Error;
use serde::Deserialize;
use crate::algorithm::entities::{Position, data::DataArray, ship_model::GaseousData};
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
    pub assignment_id: usize,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,    
    /// масса, т
    pub mass: f64,
    /// Общая масса, т
    pub volume: Option<f64>,
    /// Центр отсека, размещающего груз, м
    pub mass_shift_x: Option<f64>,
    pub mass_shift_y: Option<f64>,
    pub mass_shift_z: Option<f64>,
}
//
impl LoadGaseousData {
    pub fn data(&self) -> GaseousData {
        GaseousData {
            assignment_id:  self.assignment_id,
            assigment_type: self.assigment_type,
        //    cargo_id: self.cargo_id,
            space_id: self.space_id.clone(),
            mass: self.mass,
        }
    }
//
    pub fn mass_shift(&self) -> Result<Position, Error> {
        let error = Error::new("LoadGaseousData", "mass_shift");
        let center_x =  if let Some(x) = self.mass_shift_x {
            x
        } else {
            return Err(error.err("no mass_shift_x"));
        };
        let center_y =  if let Some(v) = self.mass_shift_y {
            v
        } else {
            return Err(error.err("no mass_shift_y"));
        };
        let center_z =  if let Some(v) = self.mass_shift_y {
            v
        } else {
            return Err(error.err("no mass_shift_z and bound_z2"));
        };
        return Ok(Position::new(center_x, center_y, center_z));        
    }
}

/// Массив данных по грузам
pub type LoadGaseousArray = DataArray<LoadGaseousData>;
//
impl LoadGaseousArray {
    pub fn data(self) -> HashMap<usize, LoadGaseousData> {
        self.data.into_iter().filter(|v| v.mass > 0.).map(|v| (v.assignment_id, v)).collect()
    }
}
