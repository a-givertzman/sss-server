use crate::{
    algorithm::entities::{
        Position, cache::Cache, model_cached::{DisplacementShape, local_cache::LocalCache, save}
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
/// Pre-calculated cache for floating position algorithm.
pub struct CompartmentCache {
    dbg: Dbg,
    cache_path: PathBuf,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    level_qnt_steps: usize,
    /// центр полного объема из бд
    center_max: Option<Position>,
    /// полный объем из бд
    volume_max: Option<f64>,
    /// максимальная высота заполнения отсека
    level_max: Option<f64>,
    ///
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<DisplacementShape>>,
    ///
    /// Cache read from `self.file_path`.
    cache: Arc<RwLock<Option<Cache<f64>>>>,
    scheduler: Scheduler,
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
        level_qnt_steps: usize,
        center_max: Option<Position>,
        volume_max: Option<f64>,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("Compartment_{compartment_id}_Cache"));
        Self {
            shape,
            heel_steps,
            trim_steps,
            level_qnt_steps,
            center_max,
            volume_max,
            level_max: None,
            cache: Arc::new(RwLock::new(None)),
            cache_path: cache_dir.as_ref().join(compartment_id),
            dbg,
            scheduler,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Return (level, center of volume)
    pub fn get(&self, heel: f64, trim: f64, volume: f64, epsilon: f64) -> Result<(f64, Position), Error> {
        let error = Error::new(self.dbg(), "get");
        self.init().map_err(|err| error.pass_with("self.init()", err))?;
        let guard = self.cache().read();  
        let cache = guard.as_ref().ok_or(error.pass("no cache"))?;
        let level_max = cache.max_value(3);
        let mut step = level_max/2.;
        let mut draught = step;
        for _ in 0..50 {
            let query = [heel, trim, draught];
            let result = cache.get(&query);
            let delta = volume - result.first().ok_or(error.pass("no result from cache.get(&query)"))?;
            if delta.abs() <= epsilon {
                return Ok((result[0], Position::new(result[1], result[2], result[3])));
            }
            step = step/2.;
            draught += step*delta.signum();
        }
        Err(error.pass("no result"))
    }
}
//
//
impl LocalCache for CompartmentCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let cache_data = super::build_cache::BuildCompartmentCache::new(
            &self.dbg,
            self.shape.clone(),
            self.heel_steps.clone(),
            self.trim_steps.clone(),
            self.level_qnt_steps,
            self.center_max,
            self.volume_max,
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
