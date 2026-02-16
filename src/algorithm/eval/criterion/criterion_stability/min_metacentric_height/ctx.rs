//! Расчет критерия минимальной метацентрической высоты
use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct MinMetacentricHeightCtx {
    pub data: CriterionData,
}
