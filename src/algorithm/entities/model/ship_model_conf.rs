use std::path::PathBuf;
use crate::algorithm::entities::model::CacheConf;
///
/// [super::ShipModel] configuration.
///
/// It can be used to wrap configuration getting from an external source.
pub struct ShipModelConf {
    ///
    /// File containing model structure (e. g. in STL format).
    pub model_path: PathBuf,
    /// Directory containing files with model additional structures (e. g. in STL format).
    pub additional_path: Option<PathBuf>,
    ///
    pub model_scale: f64,
    ///
    /// Directory containing [super::ShipModel] caches.
    pub cache_dir: PathBuf,
    ///
    /// Cache configuration.
    pub cache_conf: CacheConf,
}
