//! Промежуточные структуры для serde_json для парсинга данных отделений трюма образованных зерновыми перегородками
use crate::algorithm::entities::data::{DataArray, loads::{AssignmentType, BulkCargoType}};
use serde::Deserialize;
///
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct HoldCompartmentData {
    /// Индекс группы (трюма) 
    pub group_id: usize,
    /// Индекс помещения в группе
    pub group_start_index: usize,
    /// ID ограничивающего помещения слева
    pub left_bulkhead_space_id: Option<String>,
    /// ID ограничивающего помещения справа    
    pub right_bulkhead_space_id: Option<String>,
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
    pub mass_shift_x: Option<f64>,
    pub mass_shift_y: Option<f64>,
    pub mass_shift_z: Option<f64>,    
}
/// Массив данных отделений трюма
pub type HoldCompartmentDataArray = DataArray<HoldCompartmentData>;
