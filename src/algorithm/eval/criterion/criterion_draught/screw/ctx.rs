//! Расчет критерия заглубления винта
use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct ScrewCtx {
    pub data: Vec<CriterionData>
}
