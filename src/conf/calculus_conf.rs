use sal_sync::services::conf::ConfDuration;
use serde::Deserialize;

///
/// Configuration parameters for the calculations
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CalculusConf {
    /// Maximum time wait calculation, then return error
    #[serde(alias="max-time")]
    pub max_time: ConfDuration,
}