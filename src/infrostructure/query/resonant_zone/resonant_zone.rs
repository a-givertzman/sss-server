use serde::{
    Serialize, 
    Deserialize
};
use crate::infrostructure::query::resonant_zone::zone_id::ZoneID;
///
/// DataBase request | 
/// Save result's of [ParametricResonantZoneSpeedFilter](src/algorithm/eval/seakeeping/parametric_resonant_zone_speed_filter) 
/// or [MainResonantZoneSpeedFilter](src/algorithm/eval/seakeeping/main_resonant_zone_speed_filter)
/// or [MoveBrochingFilter](src/algorithm/eval/seakeeping/move_broching_filter)
/// or [ImpactsHighWaves](src/algorithm/eval/seakeeping/impacts_high_waves)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResonantZoneQuery {
    pub resonant_zone: Vec<(f64,f64)>,
    pub zone_id: ZoneID,
}
//
impl ResonantZoneQuery {
    ///
    /// New instance [ResonantZoneQuery]
    pub fn new(resonant_zone: Vec<(f64,f64)>, zone_id: ZoneID) -> Self {
        Self {
            resonant_zone,
            zone_id,
        }
    }
    ///
    /// Create SQL query
    pub fn sql(&self) -> String {
        let create_table = format!("
            CREATE TYPE IF NOT EXISTS zone_type AS ENUM ('Parametric', 'Main', 'Broaching', 'HighWaves');

            CREATE TABLE IF NOT EXISTS seakeeping_zones (
                id SERIAL PRIMARY KEY,
                angle FLOAT NOT NULL,
                speed FLOAT NOT NULL,
                zone_id zone_type NOT NULL,
                
                CONSTRAINT unique_angle_speed_zone UNIQUE (angle, speed, zone_id)
            );"
        );
        let insert_values = format!(
            "INSERT INTO seakeeping_zones (angle, speed) VALUES {};",
            self.resonant_zone.iter()
                .map(|(a, s)| format!("({}, {})", a, s))
                .collect::<Vec<_>>()
                .join(",")
        );
        create_table + &insert_values
    }
}