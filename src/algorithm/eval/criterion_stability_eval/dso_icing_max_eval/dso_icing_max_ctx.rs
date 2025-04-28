//! Расчет максимума диаграммы статической остойчивости с учетом обледенения
use crate::algorithm::eval::CriterionData;

#[derive(Debug, Clone)]
pub struct DSOIcingMaxCtx {
    pub data: CriterionData,
}
