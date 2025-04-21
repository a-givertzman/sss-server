use bincode::{Decode, Encode};
use crate::algorithm::entities::parameters::ParameterID;
use super::{bulk_result::BulkResult, liquid_result::LiquidResult};

///
#[derive(Debug, Clone, Decode, Encode)]
pub struct BalanceCtx {
    // Результаты расчета в виде (id, value)
    // id в соответствии с https://github.com/a-givertzman/sss/blob/35-shipmodel-fix-unit-cargo/docs/user-guide/ru/part08_stability/chapter03_parametresStability.md
    pub parameters: Vec<(ParameterID, f64)>, 
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса.
    pub bulk: Vec<BulkResult>,
    pub liquid: Vec<LiquidResult>,
}
