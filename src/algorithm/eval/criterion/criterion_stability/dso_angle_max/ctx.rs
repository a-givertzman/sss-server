//! Расчет угла, соответствующий максимуму диаграммы статической остойчивости
use crate::algorithm::eval::criterion::CriterionData;

#[derive(Debug, Clone)]
pub struct DSOAngleMaxCtx {
    pub data: Vec<CriterionData>,
}
