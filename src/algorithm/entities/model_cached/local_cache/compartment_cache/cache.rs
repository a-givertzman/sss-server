use crate::{
    algorithm::entities::{
        Bounds, Position,
        cache::Cache,
        model_cached::{
            BoundDisplacementCache, CompartmentCacheResult, DisplacementShape, Shape,
            local_cache::LocalCache, save,
        },
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
///
/// Pre-calculated cache for floating position algorithm.
pub struct CompartmentCache {
    dbg: Dbg,
    cache_dir: PathBuf,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    level_step: f64,
    /// центр полного объема из бд
    center_max: Option<Position>,
    /// полный объем из бд
    volume_max: Option<f64>,
    ///
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<DisplacementShape>>,
    ///
    /// Cache read from `self.file_path`.
    cache: Option<Cache<f64>>,
    thread_pool: Arc<ThreadPool>,
    exit: Arc<AtomicBool>,
}
//
//
impl CompartmentCache {
    ///
    /// Creates a new instance.
    /// * cache_dir - folder contains all cache files
    /// * center_max - центр полного объема из бд
    /// * volume_max - полный объем из бд
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        cache_dir: impl AsRef<Path>,
        compartment_id: String,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        level_step: f64,
        center_max: Option<Position>,
        volume_max: Option<f64>,
        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("Compartment_{compartment_id}_Cache"));
        Self {
            shape,
            heel_steps,
            trim_steps,
            level_step,
            center_max,
            volume_max,
            cache: None,
            cache_dir: cache_dir.as_ref().join(compartment_id),
            dbg,
            thread_pool,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Return (level, center of volume)
    pub fn get(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
    ) -> Result<CompartmentCacheResult, Error> {
        let error = Error::new(self.dbg(), "get");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let level_max = cache.max_value(2);
        let mut step = level_max / 2.;
        let mut level = step;
        for i in 0..=50 {
            let query = [heel, trim, level];
            let result = cache.get(&query);
            assert!(result.len() == 6);
            let delta = result
                .first()
                .ok_or(error.pass("no result from cache.get(&query)"))?
                - volume;
            if delta.abs() <= epsilon || i >= 50 {
                return Ok(CompartmentCacheResult {
                    heel,
                    trim,
                    level,
                    volume,
                    volume_center: Position::new(result[1], result[2], result[3]),
                    inertia_trans_x: result[4],
                    inertia_long_y: result[5],
                });
            }
            step = step / 2.;
            level -= step * delta.signum();
        }
        Err(error.pass(format!("no result for epsilon:{epsilon}")))
    }
    //
    pub fn build_bounded(
        &self,
        bounds: Bounds,
        level_step: f64,
    ) -> Result<BoundDisplacementCache, Error> {
        let center_x = self
            .shape
            .read()
            .center()
            .ok_or(Error::new(self.dbg.clone(), "build_bounded").err("no center point for shape"))?
            .x;
        Ok(BoundDisplacementCache::new(
            &self.dbg,
            self.shape.clone(),
            self.cache_dir.clone().join("distr"),
            level_step,
            center_x,
            bounds,
            Arc::clone(&self.thread_pool),
        ))
    }
}
//
//
impl LocalCache for CompartmentCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let (data, mut errors) = super::build_cache::BuildCompartmentCache::new(
            &self.dbg,
            self.shape.clone(),
            self.heel_steps.clone(),
            self.trim_steps.clone(),
            self.level_step,
            self.center_max,
            self.volume_max,
            Arc::clone(&self.thread_pool),
            self.exit.clone(),
        )
        .build();
        let cache = if let Some(cache) = self.cache.take() {
            cache
        } else {
            Cache::<f64>::new(&self.dbg)
        };
        if let Err(err) = cache.init(data.clone()) {
            errors.push(error.pass_with("self.cache.get_mut", err));
        }
        self.cache = Some(cache);
        if let Err(err) = save(&self.dbg, &self.cache_path(), data) {
            errors.push(error.pass_with("save data", err));
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
    fn cache_path(&self) -> PathBuf {
        self.cache_dir.clone().join("disp")
    }
    //
    fn cache(&self) -> Option<&Cache<f64>> {
        self.cache.as_ref()
    }
    //
    fn set_cache(&mut self, cache: Cache<f64>) {
        let _ = self.cache.insert(cache);
    }
}
