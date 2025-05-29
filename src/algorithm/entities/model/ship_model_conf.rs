use super::local_cache::DisplacementCacheConf;
use std::path::PathBuf;
///
/// [super::ShipModel] configuration.
///
/// It can be used to wrap configuration getting from an external source.
pub struct ShipModelConf {
    ///
    /// File containing model structure (e. g. in STEP format).
    pub model_path: PathBuf,
    ///
    pub model_scale: f64,
    ///
    /// Directory containing [super::ShipModel] caches.
    pub cache_dir: PathBuf,
    ///
    /// [super::DisplacementCache] configuration.
    pub floating_position_cache_conf: DisplacementCacheConf,
}
