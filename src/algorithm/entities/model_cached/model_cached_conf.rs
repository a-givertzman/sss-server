use std::{collections::HashMap, path::PathBuf};
use crate::algorithm::entities::Position;
///
/// [super::ModelCached] configuration.
///
/// It can be used to wrap configuration getting from an external source.
pub struct ModelCachedConf {
    /// Directory containing [super::ModelCached] caches.
    pub cache_dir: PathBuf,
    /// Directory containing model structure (e. g. in STL format).
    pub model_dir: PathBuf,
    /// Scale of model, shape will be scaled by value = 1/model_scale
    pub model_scale: f64,
    /// Waterline initial position in 3D space (midel).
    pub model_center_coord: Position,
    /// Ship length between perpendiculars
    pub ship_length_lbp: f64,
    /// Angle in degrees.
    pub heel_steps: Vec<f64>,
    /// Angle in degrees.
    pub trim_steps: Vec<f64>,
    /// Draught in meters
    pub draught_min: f64,
    pub draught_max: f64,
    /// Draught step for hull
    pub hull_draught_step: f64,
    /// Level step for compartments
    pub compartment_level_step: f64,
    /// qnt steps for compartments
    pub compartment_qnt_steps: usize,
    /// volume data for compartments
    pub compartment_data: HashMap<String, (Option<f64>, Option<Position>)>,
}
