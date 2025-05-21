use crate::algorithm::entities::Position;

///
/// [super::DisplacementCache] configuration.
#[derive(Default)]
pub struct DisplacementCacheConf {
    ///
    /// Waterline initial position in 3D space.
    pub waterline_position: Position,
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
