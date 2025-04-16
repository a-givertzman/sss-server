use crate::{algorithm::entities::parameters::ParameterID, ship_model::reply::{BulkData, LiquidData}};

///
#[derive(Debug, Clone)]
pub struct BalanceCtx {
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса.
    pub bulk: Vec<BulkData>,
    pub liquid: Vec<LiquidData>,
}
