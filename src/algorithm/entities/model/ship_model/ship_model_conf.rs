use super::local_cache::floating_position_cache::floating_position_cache_conf::FloatingPositionCacheConf;
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
    /// Directory containing [super::ShipModel] caches.
    pub cache_dir: PathBuf,
    ///
    /// [super::FloatingPositionCache] configuration.
    pub floating_position_cache_conf: FloatingPositionCacheConf,
}
