//! Расчет критерия максимум диаграммы статической остойчивости
use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct DSOMaxCtx {
    pub data: CriterionData 
}
