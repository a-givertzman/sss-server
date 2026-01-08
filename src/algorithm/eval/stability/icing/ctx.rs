use crate::algorithm::entities::Moment;
///
/// Учет обледенения судна
#[derive(Debug, Clone, PartialEq)]
pub struct IcingStabCtx {
    /// Суммарная масса льда
    pub p_ice: f64,
    /// Момент массы льда
    pub m_ice: Moment,
}
