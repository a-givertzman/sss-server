//! Расчет критерия запаса плавучести в носу
use crate::algorithm::eval::CriterionData;

#[derive(Debug, Clone)]
pub struct ReserveBuoyncyCtx {
    pub data: CriterionData
}
