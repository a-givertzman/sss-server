use serde::{Deserialize, Serialize};
use super::{
    restart_eval::RestartEvalQuery,
    resonant_zone::resonant_zone::ResonantZoneQuery,
};
///
/// List of all possible requests in Client-Server interface
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Query {
    ///
    /// Client request | Restart of calculation
    RestartEval(RestartEvalQuery),
    ///
    /// DataBase request | 
    /// Save result's of [ParametricResonantZoneSpeedFilter](src/algorithm/eval/seakeeping/parametric_resonant_zone_speed_filter) 
    /// or [MainResonantZoneSpeedFilter](src/algorithm/eval/seakeeping/main_resonant_zone_speed_filter)
    /// or [MoveBrochingFilter](src/algorithm/eval/seakeeping/move_broching_filter)
    /// or [ImpactsHighWaves](src/algorithm/eval/seakeeping/impacts_high_waves)
    ParametricZone(ResonantZoneQuery),
}
