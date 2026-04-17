use crate::algorithm::entities::Moment;
///
/// Учет намокания палубного груза.  
#[derive(Debug, Clone)]
pub struct WettingCtx {
    /// Суммарная масса
    pub mass: f64,
    /// Момент массы намокания палубного груза
    pub moment: Moment,
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,
}
