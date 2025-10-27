//! Промежуточные структуры для serde_json для парсинга данных груза
use std::collections::HashMap;

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
    pub assigned_id: usize,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,    
    /// масса, т
    pub mass: f64,
    /// Общая масса, т
    pub volume: Option<f64>,
    /// Центр отсека, размещающего груз, м
    pub mass_shift: Option<Position>,
}
//
impl LoadGaseousData {
    pub fn data(&self) -> GaseousData {
        GaseousData {
            assigned_id:  self.assigned_id,
            assigment_type: self.assigment_type,
        //    cargo_id: self.cargo_id,
            space_id: self.space_id.clone(),
            mass: self.mass,
        }
    }
}

/// Массив данных по грузам
pub type LoadGaseousArray = DataArray<LoadGaseousData>;
//
impl LoadGaseousArray {
    pub fn data(self) -> HashMap<usize, LoadGaseousData> {
        self.data.into_iter().filter(|v| v.mass > 0.).map(|v| (v.assigned_id, v)).collect()
    }
}
