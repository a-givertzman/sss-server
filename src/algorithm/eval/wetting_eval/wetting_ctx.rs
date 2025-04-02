//! Учет намокания груза
use crate::algorithm::entities::Position;
///
#[derive(Debug, Clone)]
pub struct WettingCtx {
    /// Суммарная масса
    pub mass: f64,
    /// Смещение центра массы
    pub mass_shift: Position,
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,
}
