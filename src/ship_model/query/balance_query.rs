use bincode::{Decode, Encode};
use crate::algorithm::entities::Moment;
use super::{bulk_data::BulkData, liquid_data::LiquidData};

///
/// Структура для ввода данных расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone, Decode, Encode)]
pub struct BalanceQuery {
    /// Суммарная масса корпуса, всех грузов и обледенения с намоканием
    pub mass_sum: f64,
    /// Сумарный момент за вычетом смещяемых и насыпных груов
    pub moment_const: Moment,
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса, считается в модели
    pub bulk: Vec<BulkData>,
    pub liquid: Vec<LiquidData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>,
}
