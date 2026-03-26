use serde::{
    Serialize, 
    Deserialize
};
///
/// ID of [resonant zones](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZoneID {
    /// [ParametricResonantZoneSpeedFilter](src/algorithm/eval/seakeeping/parametric_resonant_zone_speed_filter) 
    Parametric,
    /// [MainResonantZoneSpeedFilter](src/algorithm/eval/seakeeping/main_resonant_zone_speed_filter)
    Main,
    /// [MoveBrochingFilter](src/algorithm/eval/seakeeping/move_broching_filter)
    Broching,
    /// [ImpactsHighWaves](src/algorithm/eval/seakeeping/impacts_high_waves)
    HighWaves,
}