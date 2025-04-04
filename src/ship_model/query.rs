///
/// Queries to the `ShipModel`
///
use crate::algorithm::entities::Moment;
///
#[derive(Debug, Clone)]
pub struct LiquidData {
    pub cargo_id: usize, // ID груза
    pub space_id: usize, // ID помещения
    pub mass: f64,
    pub volume: f64,
}
///
#[derive(Debug, Clone)]
pub struct BulkData {
    pub cargo_id: usize, // ID груза
    pub space_id: usize, // ID помещения
    pub mass: f64,
    pub volume: f64,
}
/// Структура для ввода данных расчета баланса судна. Содержит массу судна, грузов и положение
/// зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceSrcData {
    // Суммарная масса корпуса, всех грузов и обледенения с намоканием
    pub mass_sum: f64,
    // Сумарный момент за вычетом смещяемых и насыпных груов
    pub moment_const: Moment,
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса, считается в модели
    pub bulk: Vec<BulkData>,
    pub liquid: Vec<LiquidData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>,
}
//
#[derive(Debug)]
pub enum Query {
    Bounds,
    AreasStrength,
    ComputeBalance(BalanceSrcData),
}
