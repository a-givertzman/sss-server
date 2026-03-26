//! Расчет критерия запаса плавучести в носу
use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct ReserveBuoyncyCtx {
    pub data: CriterionData
}
