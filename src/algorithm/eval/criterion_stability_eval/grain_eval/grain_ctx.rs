//! Расчет критерия при перевозки навалочных смещаемых грузов
use crate::algorithm::eval::CriterionData;

#[derive(Debug, Clone)]
pub struct GrainCtx {
    pub data: Vec<CriterionData> 
}
