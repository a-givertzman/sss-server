use bincode::{Decode, Encode};
use super::{bulk_result::BulkResult, liquid_result::LiquidResult, gaseous_result::GaseousResult};

///
#[derive(Debug, Clone, Decode, Encode)]
pub struct StrengthBalanceCtx {
    /// Сыпучий груз для которого центр массы и распределение зависит от 
    /// объема
    pub bulk: Vec<BulkResult>,
    /// Жидкий груз, для которого центр массы и распределение зависит от 
    /// объема и положения корпуса
    pub liquid: Vec<LiquidResult>,
    /// распределение массы газообразных грузов по шпациям 
    pub gaseous: Vec<GaseousResult>,
    /// весовое водоизмещение по шпациям, м^3
    pub displacement_distr: Vec<f64>,
}
