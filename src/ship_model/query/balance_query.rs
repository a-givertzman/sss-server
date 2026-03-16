use bincode::{Decode, Encode};
use crate::algorithm::entities::{Moment, Position};
use super::{bulk_data::BulkData, liquid_data::LiquidData};

///
/// Структура для ввода данных расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone, Decode, Encode)]
pub struct BalanceQuery {
    /// Плотность забортной воды
    pub water_density: f64,
    /// Суммарная масса корпуса, всех грузов и обледенения с намоканием
    pub mass_sum: f64,
    /// Смещение суммарной массы.
    /// TODO: пересчитывать смещение с учетом смещяемых и насыпных грузов
    pub mass_shift: Position,
    /// Сумарный момент за вычетом смещяемых и насыпных грузов
    pub moment_const: Moment,
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса, считается в модели
    pub bulk: Vec<BulkData>,
    pub liquid: Vec<LiquidData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>,
}
