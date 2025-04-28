//! Расчет критерия погоды К

use crate::algorithm::eval::CriterionData;
#[derive(Debug, Clone)]
pub struct WheatherCtx {
    pub data: CriterionData, 
}
