use sal_sync::services::entity::error::str_err::StrErr;

use crate::algorithm::entities::{parameters::ParameterID, Bounds, Position};
///
#[derive(Debug, Clone)]
pub struct LiquidData {
    pub cargo_id: usize, // ID груза
    pub space_id: usize, // ID помещения
    pub mass_shift: Position, // смещение центра массы
    pub long_moment_of_inertia: f64, // продольный момент свободной поверхности жидкости
    pub trans_moment_of_inertia: f64, // поперечный момент свободной поверхности жидкости
    pub mass_values: Vec<(usize, f64)>, // Распределение массы по шпациям, (index, value)
}
///
#[derive(Debug, Clone)]
pub struct BulkData {
    pub cargo_id: usize, // ID груза
    pub space_id: usize, // ID помещения
    pub mass_shift: Position, // смещение центра массы
    pub mass_values: Vec<(usize, f64)>, // Распределение массы по шпациям, (index, value)
}
/// Структура результатов расчета баланса судна
#[derive(Debug, Clone)]
pub struct BalanceResultData {
    // Результаты расчета в виде (id, value)
    // id в соответствии с https://github.com/a-givertzman/sss/blob/35-shipmodel-fix-unit-cargo/docs/user-guide/ru/part08_stability/chapter03_parametresStability.md
    pub parameters: Vec<(ParameterID, f64)>, 
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса.
    pub bulk: Vec<BulkData>,
    pub liquid: Vec<LiquidData>,
}
///
/// Replies from the `ShipModel`
#[derive(Debug)]
pub enum Reply {

    Bounds(Bounds),
    AreasStrength(Result<(Vec<f64>, Vec<f64>), StrErr>),
    ComputeBalance(Result<BalanceResultData, StrErr>),
}
