//! Результаты расчета критериев проверки остойчивости судна

use super::CriterionData;
#[derive(Debug, Clone)]
pub struct CriterionStabilityCtx {
    /// 
    pub data: Vec<CriterionData>,
}
