//! Промежуточные структуры для serde_json для парсинга данных груза
use std::collections::HashMap;

use super::{AssignmentType, BulkCargoType};
use crate::algorithm::entities::{Position, data::DataArray, ship_model::BulkData};
use serde::{Deserialize, Serialize};
///
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadBulkData {
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
    /// Тип сыпучего груза
    pub cargo_type: BulkCargoType,
    /// масса, т
    pub mass: f64,
    /// Признак смещаемости груза. При его размещении применяются правила перевозки зерна 
    pub shiftable: bool,
    /// Средний удельный погрузочный объем, м^3/т
    pub stowage_factor: Option<f64>,
    /// Обьем, м^3
    pub volume: Option<f64>,
    /// Центр отсека, размещающего груз, м
    pub mass_shift: Option<Position>,
}
//
impl LoadBulkData {
    pub fn data(&self) -> Option<BulkData> {
        if self.mass <= 0. {
            return None;
        }
        let volume = if let Some(volume) = self.volume {
            volume
        } else {
            if let Some(stowage_factor) = self.stowage_factor {
                self.mass * stowage_factor
            } else {
                return None;
            }
        };
        Some(BulkData {
            assignment_id:  self.assignment_id,
            assigment_type: self.assigment_type,
        //    cargo_id: self.cargo_id,
            space_id: self.space_id.clone(),
            mass: self.mass,            
            volume,
            shiftable: self.shiftable,
        })
    }
}
/*
impl std::fmt::Display for LoadBulkData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoadBulkData(space_id:{} space_name:{} cargo_id:{} cargo_name:{} assignment_id:{}
                assigment_type:{} cargo_type:{}, mass:{}, stowage_factor:{} volume:{} )",
            self.space_id,
            self.space_name,
            self.cargo_id,
            self.cargo_name,
            self.assignment_id,
            self.assigment_type,
            self.cargo_type,
            self.mass.unwrap_or(0.),
            self.stowage_factor.unwrap_or(0.),
            self.volume.unwrap_or(0.),
        )
    }
}*/
/// Массив данных по грузам
pub type LoadBulkArray = DataArray<LoadBulkData>;
//
impl LoadBulkArray {
    pub fn data(self) -> HashMap<usize, LoadBulkData> {
        self.data.into_iter().filter(|v| v.mass > 0.).map(|v| (v.assignment_id, v)).collect()
    }
}
