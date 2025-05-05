//! Расчет критерия осадки по грузовой марке
use crate::algorithm::eval::CriterionData;

#[derive(Debug, Clone)]
pub struct LoadLineCtx {
    pub data: Vec<CriterionData> 
}
