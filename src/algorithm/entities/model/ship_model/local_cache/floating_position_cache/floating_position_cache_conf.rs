///
/// [super::FloatingPositionCache] configuration.
#[derive(Default)]
pub struct FloatingPositionCacheConf {
    ///
    /// Waterline initial position in 3D space.
    pub waterline_position: [f64; 3],
    ///
    /// Angle in degrees.
    pub heel_steps: Vec<f64>,
    ///
    /// Angle in degrees.
    pub trim_steps: Vec<f64>,
    ///
    /// TODO: clarify units.
    pub draught_steps: Vec<f64>,
}
