use crate::algorithm::entities::Moment;
///
/// Учет обледенения судна
#[derive(Debug, Clone, PartialEq)]
pub struct IcingStabCtx {
    /// Суммарная масса льда
    pub mass: f64,
    /// Момент массы льда
    pub moment: Moment,
}
