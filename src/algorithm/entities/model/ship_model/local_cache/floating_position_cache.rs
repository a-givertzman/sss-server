mod calculated_floating_position_cache;
pub mod floating_position_cache_conf;
//
use super::{super::ModelTree, Cache, LocalCache};
use calculated_floating_position_cache::CalculatedFloatingPositionCache;
use floating_position_cache_conf::FloatingPositionCacheConf;
use sal_3dlib::topology::shape::{
    face::Face,
    vertex::Vertex,
    wire::{Polygon, Wire},
};
use sal_sync::services::{
    entity::{dbg_id::DbgId, error::str_err::StrErr},
    service::service_handles::ServiceHandles,
};
use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
};
///
/// Pre-calculated cache for floating position algorithm.
///
/// See [FloatingPositionCacheConf] for more details about the fields.
pub(in super::super) struct FloatingPositionCache<A> {
    dbgid: DbgId,
    file_path: PathBuf,
    model_keys: Vec<String>,
    waterline_position: [f64; 3],
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    ///
    /// Model representation used for cache calculation.
    model_tree: ModelTree<A>,
    ///
    /// Cache read from `self.file_path`.
    cache: Cache<f64>,
}
//
//
impl<A> FloatingPositionCache<A> {
    //
    //
    const KEY: &'static str = "floating_position_cache";
    ///
    /// Creates a new instance.
    /// - path - folder contains all cache files
    pub(in super::super) fn new(
        parent: &DbgId,
        model_tree: ModelTree<A>,
        path: impl AsRef<Path>,
        conf: FloatingPositionCacheConf,
    ) -> Self {
        let dbgid = DbgId::with_parent(parent, "FloatingPositionCache");
        let file_path = path.as_ref().join(Self::KEY);
        Self {
            model_tree,
            model_keys: vec![],
            heel_steps: conf.heel_steps,
            waterline_position: conf.waterline_position,
            trim_steps: conf.trim_steps,
            draught_steps: conf.draught_steps,
            cache: Cache::new(&dbgid, &file_path),
            file_path,
            dbgid,
        }
    }
    ///
    /// Creates a waterline object in 3D space centered at `self.waterline_position`.
    ///
    /// The result object is used for calculating cache algorithm (see [FloatingPositionCache::calculate]).
    fn create_waterline<T>(&self) -> Result<Face<T>, StrErr> {
        let dbgid = DbgId(format!("{}.create_waterline_model", self.dbgid));
        let [x, y, z] = self.waterline_position;
        // dynamic range could be built based on bounding box of target element behind self.model_keys,
        // but now reserve big enough offsets, which should work with most elements
        let dx = 1000.0;
        let dy = 1000.0;
        //
        match Wire::polygon(
            [
                Vertex::new([x + dx, y + dy, z]),
                Vertex::new([x - dx, y + dy, z]),
                Vertex::new([x - dx, y - dy, z]),
                Vertex::new([x + dx, y - dy, z]),
            ],
            true,
        ) {
            Ok(ref polygon) => Face::try_from(polygon).map_err(|why| {
                StrErr(format!(
                    "{} | Failed creating Face from *polygon*: {}",
                    dbgid, why
                ))
            }),
            Err(why) => Err(StrErr(format!(
                "{} | Failed creating *polygon* from Wire: {}",
                dbgid, why
            ))),
        }
    }
}
//
//
impl<A: Clone + Send + 'static> LocalCache for FloatingPositionCache<A> {
    ///
    /// See [CalculatedFloatingPositionCache] for details.
    fn calculate(
        &self,
        exit: Arc<AtomicBool>,
    ) -> Result<ServiceHandles<Result<(), StrErr>>, StrErr> {
        CalculatedFloatingPositionCache::new(
            &self.dbgid,
            self.file_path.clone(),
            self.model_tree
                .iter()
                .filter_map(|(shape_key, shape)| {
                    self.model_keys.contains(shape_key).then_some(shape)
                })
                .cloned()
                .collect(),
            self.create_waterline()?,
            self.heel_steps.clone(),
            self.trim_steps.clone(),
            self.draught_steps.clone(),
            exit,
        )
        .build()
    }
    ///
    /// See [Cache::get] for details.
    fn get(&self, approx_vals: &[Option<f64>]) -> Option<Vec<Vec<f64>>> {
        self.cache.get(approx_vals)
    }
    //
    //
    fn reload(&mut self) {
        self.cache = Cache::new(&self.dbgid, &self.file_path);
    }
}
