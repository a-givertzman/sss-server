//! Расчет критерия максимум диаграммы статической остойчивости
use crate::algorithm::eval::CriterionData;

#[derive(Debug, Clone)]
pub struct DSOMaxCtx {
    pub data: CriterionData 
}
