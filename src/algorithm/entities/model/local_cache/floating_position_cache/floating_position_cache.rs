use crate::{algorithm::entities::{cache::Cache, model::{local_cache::LocalCache, ModelTree}}, kernel::types::RwLock};
use sal_3dlib::topology::shape::{
    face::Face,
    vertex::Vertex,
    wire::{Polygon, Wire},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::Scheduler;
use std::{
    path::{Path, PathBuf},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
};

use super::{build_floating_position_cache::BuildFloatingPositionCache, FloatingPositionCacheConf};
///
/// Pre-calculated cache for floating position algorithm.
///
/// See [FloatingPositionCacheConf] for more details about the fields.
pub struct FloatingPositionCache {
    dbg: Dbg,
    path: PathBuf,
    //    model_keys: Vec<String>,
    waterline_position: [f64; 3],
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    ///
    /// Model representation used for cache calculation.
    model_tree: ModelTree,
    ///
    /// Cache read from `self.file_path`.
    cache: Arc<RwLock<Cache<f64>>>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl FloatingPositionCache {
    //
    //
    const KEY: &'static str = "floating_position_cache";
    ///
    /// Creates a new instance.
    /// - path - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        model_tree: ModelTree,
        path: impl AsRef<Path>,
        conf: FloatingPositionCacheConf,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, "FloatingPositionCache");
        let path = path.as_ref().join(Self::KEY);
        Self {
            model_tree,
            //         model_keys: vec![],
            heel_steps: conf.heel_steps,
            waterline_position: conf.waterline_position,
            trim_steps: conf.trim_steps,
            draught_steps: conf.draught_steps,
            cache: Arc::new(RwLock::new(Cache::new(&dbg, &path))),
            path,
            dbg,
            scheduler,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Creates a waterline object in 3D space centered at `self.waterline_position`.
    ///
    /// The result object is used for calculating cache algorithm (see [FloatingPositionCache::calculate]).
    pub fn create_waterline<T>(&self) -> Result<Face<T>, Error> {
        let error = Error::new(&self.dbg, "create_waterline");
        let [x, y, z] = self.waterline_position;
        let (x, y, z) = (x*1000., y*1000., z*1000.); // m to mm
        // dynamic range could be built based on bounding box of target element behind self.model_keys,
        // but now reserve big enough offsets, which should work with most elements
        let dx = 1000000.0;  
        let dy = 1000000.0;
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
            Ok(ref polygon) => Face::try_from(polygon)
                .map_err(|why| error.pass_with("Failed creating Face from *polygon*:", why)),
            Err(why) => {
                Err(error.pass_with("Failed creating *polygon* from Wire", why.to_string()))
            }
        }
    }
    ///
    /// See [BuildFloatingPositionCache] for details.
    fn calculate(&self) -> Vec<Error> {
        let waterline = match self.create_waterline() {
            Ok(waterline) => waterline,
            Err(err) => {
                return vec![Error::new(&self.dbg, "calculate").pass_with("waterline", err)];
            }
        };
        let model_tree = self.model_tree.clone();
        let model_tree = match model_tree.load() {
            Ok(model_tree) => model_tree,
            Err(err) => {
                return vec![Error::new(&self.dbg, "calculate").pass_with("model_tree", err)];
            }
        };
        BuildFloatingPositionCache::new(
            &self.dbg,
            self.path.clone(),
            model_tree.iter().map(|(_, shape)| shape).cloned().collect(),
            /* TODO зачем этот фильтр?
                    .iter()
                       .filter_map(|(shape_key, shape)| {
                            self.model_keys.contains(shape_key).then_some(shape)
                        })
                       .cloned()
                        .collect(),
            */
            waterline,
            self.heel_steps.clone(),
            self.trim_steps.clone(),
            self.draught_steps.clone(),
            self.scheduler.clone(),
            self.exit.clone(),
        )
        .build()
    }
}
//
//
impl LocalCache for FloatingPositionCache {
    ///
    /// See [Cache::get] for details.
    fn get(&self, approx_vals: &[Option<f64>]) -> Option<Vec<Vec<f64>>> {
        self.cache.read().get(approx_vals)
    }
    //
    //
    fn rebuild(&self) -> Result<(), Error> {
        self.exit.store(false, Ordering::SeqCst);
        match self.calculate().first() {
            Some(err) => Err(Error::new(&self.dbg, "rebuild").pass(err.to_owned())),
            None => {
                *self.cache.write() = Cache::new(&self.dbg, &self.path);
                Ok(())
            }
        }
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst)
    }
}
