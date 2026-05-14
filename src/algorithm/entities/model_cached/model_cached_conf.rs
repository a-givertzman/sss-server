use std::path::PathBuf;
///
/// [super::ModelCached] configuration.
///
/// It can be used to wrap configuration getting from an external source.
pub struct ModelCachedConf {
    /// Directory containing [super::ModelCached] caches.
    pub cache_dir: PathBuf,
    /// Hull
    /// Waterline initial position in 3D space (midel).
    pub model_x: f64,
    /// Ship length between perpendiculars
    pub ship_length_lbp: f64,
    /// Minimal draught
    pub draught_min: f64,
    /// Draught in meters for calculation
    pub hull_draught_min: f64,
    pub hull_draught_max: f64,
    /// Draught step for hull
    pub hull_draught_step: f64, 
    /// Compartments
    /// Level steps for compartments
    pub compartment_level_step_qnt: usize,    
    /// Level step for compartments
    pub bounds_level_step: f64,
    /// Angles for DSO, degree
    pub dso_angles: Vec<f64>,   
}
