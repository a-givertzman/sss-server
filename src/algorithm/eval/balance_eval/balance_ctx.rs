use crate::{algorithm::entities::parameters::ParameterID, ship_model::reply::{BulkData, LiquidData}};

///
#[derive(Debug, Clone)]
pub struct BalanceCtx {
    // Результаты расчета в виде (id, value)
    // id в соответствии с https://github.com/a-givertzman/sss/blob/35-shipmodel-fix-unit-cargo/docs/user-guide/ru/part08_stability/chapter03_parametresStability.md
    pub parameters: Vec<(ParameterID, f64)>, 
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса.
    pub bulk: Vec<BulkData>,
    pub liquid: Vec<LiquidData>,
}
