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
    /// номера поврежденных помещений
    pub damaged_compartment: Vec<String>,
    /// точность расчета
    pub epsilon: f64,
}
