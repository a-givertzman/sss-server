//! Расчет критерия ускорения 𝐾∗
use crate::algorithm::eval::CriterionData;

#[derive(Debug, Clone)]
pub struct AccelerationCtx {
    pub data: CriterionData 
}
