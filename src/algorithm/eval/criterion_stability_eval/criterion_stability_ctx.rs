//! Результаты расчета критериев проверки остойчивости судна
#[derive(Debug, Clone)]
pub struct CriterionStabilityCtx {
    /// 
    pub criterion: Vec<CriterionData>,
}
