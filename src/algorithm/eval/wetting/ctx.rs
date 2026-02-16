//! Учет намокания палубного груза
use crate::algorithm::entities::Moment;
///
#[derive(Debug, Clone)]
pub struct WettingCtx {
    /// Суммарная масса
    pub mass: f64,
    /// Момент массы намокания палубного груза
    pub moment: Moment,
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,
}
