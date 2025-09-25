use crate::algorithm::entities::{Position, ship_model::{BulkData, LiquidData}};

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct LoadsCtx {
    /// Масса 
    pub mass_const: f64,
    pub mass_bulk: f64,
    pub mass_liquid: f64,
    pub mass_unit: f64,
    pub mass_gaseous: f64,
    /// Смещение центра масс
    pub shift_const: Position,
    pub shift_unit: Position,
    pub shift_gaseous: Position,
    pub shift_liquid: Position,
    pub shift_bulk: Position,
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса, считается в модели
    pub bulk: Vec<BulkData>,
    pub liquid: Vec<LiquidData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>,
}
