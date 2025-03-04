use serde::{Deserialize, Serialize};
///
/// Represents `alternarive lifting device`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AltLiftDevice {
    /// value of alternarive lifting device name 
    pub name: String,
    /// value of alternarive lifting device weight 
    pub weight: f64,
}