//! Результаты расчета критериев посадки судна

use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct CriterionDraughtCtx {
    /// 
    pub data: Vec<CriterionData>,
}
