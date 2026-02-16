//! Расчет критерия при перевозки навалочных смещаемых грузов
use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct GrainCtx {
    pub data: Vec<CriterionData> 
}
