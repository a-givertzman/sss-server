//! Расчет критерия ускорения 𝐾∗
use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct AccelerationCtx {
    pub data: CriterionData 
}
