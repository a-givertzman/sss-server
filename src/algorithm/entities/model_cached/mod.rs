//!
//! The representation of the ship in terms of its 3D elements.
//
mod local_cache;
mod model_cached_conf;
mod model_cached;
mod windage_area;
mod draught;
mod shape;
mod test;

use std::collections::HashMap;

pub(crate) use local_cache::*;
pub use model_cached_conf::*;
pub(crate) use model_cached::*;
pub(crate) use windage_area::*;
pub(crate) use draught::*;
pub use shape::*;

use crate::algorithm::entities::Moment;
///
/// Структура для ввода данных расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceQuery {
    /// Плотность забортной воды
    pub water_density: f64,
    /// масса судна порожнем и грузов размещенных на судне: 
    /// генерального груза (unitCargoAssignment), контейнеров (containerCargoAssignment), 
    /// газообразного груза (gaseousCargoAssignment), массы обледенения и намокания;
    pub mass_const: f64,
    /// Сумарный момент за вычетом смещяемых и насыпных грузов
    pub moment_const: Moment,
    /// навалочный груз
    pub bulk: Vec<BulkData>,
    /// жидкий груз
    pub liquid: Vec<LiquidData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>,
    /// номера поврежденных помещений, TODO - только для аварийного расчета
//    pub damaged_compartment: Vec<String>,
    /// точность расчета
    pub epsilon: f64,
}
///
/// Груз, для которого центр массы и распределение зависит от 
/// объема и/или положения корпуса, считается в модели
#[derive(Debug, Clone)]
pub struct BulkData {
    pub cargo_id: usize, // ID груза
    pub space_id: String, // ID помещения
    pub mass: f64,
    pub volume: f64,
}
///
/// Груз, для которого центр массы и распределение зависит от 
/// объема и/или положения корпуса, считается в модели
#[derive(Debug, Clone)]
pub struct LiquidData {
    pub cargo_id: usize, // ID груза
    pub space_id: String, // ID помещения
    pub mass: f64,
    pub volume: f64,
}
///
#[derive(Debug, Clone)]
pub struct BoundArea {
    pub v: Vec<f64>,
    pub h: Vec<f64>,
}
///
/// Структура для данных результата расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceResult {
    heel: f64,
    trim: f64,
    draught_mid: f64, // осадка на миделе
    draught_bow: f64, // осадка на носовом перпендикуляре
    draught_stern: f64, // осадка на кормовом перпендикуляре
    draught_mean: f64, // средняя осадка (в центре тяжести ватерлинии)
    center_waterline_shift: f64, // Отстояние центра тяжести ватерлинии по длине от миделя    
    volume: f64,
    displacement: Vec<f64>, // распределение осадки по шпациям
    gaseous: HashMap<String, Vec<f64>>, // распределение массы газообразных грузов по шпациям 
    bulk: HashMap<String, Vec<f64>>,    // распределение массы сыпучих грузов по шпациям  
    liquid: HashMap<String, Vec<f64>>,  // распределение массы жидких грузов по шпациям  
}





