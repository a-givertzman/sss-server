pub mod grain_moment;
pub mod ship_model;
pub mod stability_result;

use crate::algorithm::entities::{Bounds, Moment, data::loads::{AssignmentType, LiquidCargoType}};

///
/// Структура для ввода данных расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceStrengthQuery {
    ///
    pub trim: f64,
    pub draught: f64,
    /// Плотность забортной воды
    pub water_density: f64,
    /// Распределение массы судна порожнем и грузов размещенных на судне:
    /// генерального груза (unitCargoAssignment), контейнеров (containerCargoAssignment),
    /// массы обледенения и намокания;
    pub distr_static: Vec<f64>,
    /// навалочный груз
    pub bulk: Vec<BulkData>,
    /// жидкий груз
    pub liquid: Vec<LiquidData>,
    /// газообразный груз
    pub gaseous: Vec<GaseousData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>, // TODO сейчас не учитываются, добавить в расчет для отсеков
    //    /// номера поврежденных помещений, TODO - только для аварийного расчета
    //    pub damaged_compartment: Vec<String>,
    /// точность расчета
    pub epsilon: f64,
    /// шпации разбиения
    pub bounds: Bounds,
}
///
/// Структура для ввода данных расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceStabilityQuery {
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
    pub grain_bulkhead: Vec<f64>, // TODO сейчас не учитываются, добавить в расчет для отсеков
    //    /// номера поврежденных помещений, TODO - только для аварийного расчета
    //    pub damaged_compartment: Vec<String>,
    /// точность расчета
    pub epsilon: f64,
}
///
/// Груз, для которого центр массы и распределение зависит от
/// объема и/или положения корпуса, считается в модели
/// Сыпучий груз
#[derive(Debug, Clone)]
pub struct BulkData {    
    pub assigned_id: usize,// ID assigned
    pub assigment_type: AssignmentType,  // Тип назначения груза
    pub space_id: String, // ID помещения
    pub mass: f64,
    pub volume: f64,
    pub shiftable: bool,
}
///
/// Жидкий груз
#[derive(Debug, Clone)]
pub struct LiquidData {
    pub assigned_id: usize,// ID assigned
    pub assigment_type: AssignmentType,  // Тип назначения груза
    pub space_id: String, // ID помещения    
    pub cargo_type: LiquidCargoType, // Тип жидкого груза
    pub mass: f64,
    pub volume: f64,
}
///
/// Газообразный груз
/// Для него распределение не зависит от обьема или массы,
/// считаем распределение по отсеку при максимальном объеме отсека
#[derive(Debug, Clone)]
pub struct GaseousData {
    pub assigned_id: usize, // ID assigned
    pub assigment_type: AssignmentType,  // Тип назначения груза
    pub space_id: String, // ID помещения
    pub mass: f64,
}
///
#[derive(Debug, Clone)]
pub struct BoundArea {
    pub v: Vec<f64>,
    pub h: Vec<f64>,
}