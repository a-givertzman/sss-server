//! Расчет максимума диаграммы статической остойчивости для лесовозов

use crate::algorithm::eval::CriterionData;
#[derive(Debug, Clone)]
pub struct DSOTimberMaxCtx {
    pub data: CriterionData,
}
