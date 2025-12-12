//! Результаты расчета критериев проверки остойчивости судна

use crate::algorithm::eval::CriterionData;
#[derive(Debug, Clone)]
pub struct CriterionStabilityCtx {
    /// 
    pub data: Vec<CriterionData>,
}
