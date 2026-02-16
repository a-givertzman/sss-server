//! Расчет критерия площади под диаграммой статической остойчивости
use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct DSOAreaCtx {
    pub data: Vec<CriterionData> 
}
