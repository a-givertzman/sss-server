//! Расчет угла, соответствующий максимуму диаграммы статической остойчивости

use crate::algorithm::eval::CriterionData;
#[derive(Debug, Clone)]
pub struct DSOAngleMaxCtx {
    pub data: Vec<CriterionData>,
}
