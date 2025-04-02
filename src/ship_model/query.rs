///
/// Queries to the `ShipModel`
///
use crate::algorithm::entities::Moment;
///
#[derive(Debug, Clone)]
pub struct LiquidData {
    pub space_id: usize,
    pub mass: f64,
    pub volume: f64,
}
///
#[derive(Debug, Clone)]
pub struct BulkData {
    pub space_id: usize,
    pub mass: f64,
    pub volume: f64,
}
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
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
}
//
#[derive(Debug)]
pub enum Query {
    AreasStrength,
    ComputeBalance(BalanceSrcData),
}
