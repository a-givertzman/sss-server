use std::path::PathBuf;
use crate::algorithm::entities::model_cached::CacheConf;
///
/// [super::ModelCached] configuration.
///
/// It can be used to wrap configuration getting from an external source.
pub struct ModelCachedConf {
    /// File containing model structure (e. g. in STL format).
    pub model_path: PathBuf,
    /// Directory containing files with model additional structures (e. g. in STL format).
    pub additional_path: Option<PathBuf>,
    /// Directory containing files with model compartments (e. g. in STL format).
    pub compartment_path: PathBuf,
    /// qnt steps for compartments level
    pub compartment_level_steps_qnt: usize,
    /// Scale of model, shape will be scaled by wvalue = 1/model_scale
    pub model_scale: f64,
    /// Directory containing [super::ModelCached] caches.
    pub cache_dir: PathBuf,
    /// Cache configuration.
    pub cache_conf: CacheConf,
}
