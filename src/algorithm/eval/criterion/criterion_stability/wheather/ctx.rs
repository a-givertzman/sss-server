//! Расчет критерия погоды К
use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct WheatherCtx {
    pub data: CriterionData, 
}
