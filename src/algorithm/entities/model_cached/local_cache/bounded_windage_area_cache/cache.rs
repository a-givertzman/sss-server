use crate::{
    algorithm::entities::{
        cache::Cache,
        model_cached::{local_cache::LocalCache, save, AreaShape, Shape},
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::Scheduler;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
///
/// Pre-calculated cache
pub struct BoundedAreaCache {
    dbg: Dbg,
    cache_path: PathBuf,
    /// Draught in meters
    draught_min: f64,
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<AreaShape>>,
    /// Cache read from `self.file_path`.
    cache: Arc<RwLock<Option<Cache<f64>>>>,
    exit: Arc<AtomicBool>,
}
//
//
impl BoundedAreaCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<AreaShape>>,
        cache_dir: impl AsRef<Path>,
        draught_min: f64,
    ) -> Self {
        let dbg = Dbg::new(parent, "BoundedAreaCache");
        let path = cache_dir.as_ref().join("bounded_area_cache");
        Self {
            shape,
            draught_min,
            cache: Arc::new(RwLock::new(None)), 
            cache_path: path,
            dbg,
        }
    }
}
//
//
impl LocalCache for BoundedAreaCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let cache_data = super::build_cache::BuildBoundedAreaCache::new(
            &self.dbg,
            self.shape.clone(),
            self.trim_steps.clone(),
            self.draught_min,
            self.draught_step,
            self.scheduler.clone(),
            self.exit.clone(),
        )
        .build();
        let data: Vec<_> = cache_data.iter().filter_map(|v| v.clone().ok()).collect();
        let mut errors: Vec<_> = cache_data.into_iter().filter_map(|v| v.err()).collect();
        if let Some(mut guard) = self.cache.try_write() {
            let cache = if let Some(cache) = guard.take() {
                cache
            } else {
                Cache::<f64>::new(&self.dbg)
            };
            if let Err(err) = cache.init(data.clone()) {
                errors.push(error.pass_with("self.cache.get_mut", err));
            }
            let _ = guard.insert(cache);
            if let Err(err) = save(&self.dbg, &self.cache_path, data) {
                errors.push(error.pass_with("save data", err));
            }
        } else {
            errors.push(error.err("self.cache.get_mut error: no cache"));
        }
        errors
    }
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst)
    }
    //
    fn clear_exit(&self) {
        self.exit.store(false, Ordering::SeqCst)
    }
    //
    fn dbg(&self) -> &Dbg {
        &self.dbg
    }
    //
    fn cache_path(&self) -> &PathBuf {
        &self.cache_path
    }
    //
    fn cache(&self) -> &crate::kernel::types::Arc<RwLock<Option<Cache<f64>>>> {
        &self.cache
    }
}
