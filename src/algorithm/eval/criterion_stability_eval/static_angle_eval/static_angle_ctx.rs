//! Расчет критерия статического угла крена от действия постоянного ветра
use crate::algorithm::eval::CriterionData;

#[derive(Debug, Clone)]
pub struct StaticAngleCtx {
    pub data: CriterionData,
}
