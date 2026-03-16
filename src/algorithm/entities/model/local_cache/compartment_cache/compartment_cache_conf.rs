use crate::algorithm::entities::Position;

///
/// [super::CompartmentCache] configuration.
#[derive(Default)]
pub struct CompartmentCacheConf {
    ///
    /// Waterline initial position in 3D space.
    pub center_coord: Position,
    ///
    /// Angle in degrees.
    pub heel_steps: Vec<f64>,
    ///
    /// Angle in degrees.
    pub trim_steps: Vec<f64>,
    ///
    /// Draught in meters
    pub draught_steps: Vec<f64>,
}
