use crate::{
    algorithm::entities::{
        Position, cache::Cache, model_cached::{DisplacementShape, draught, local_cache::LocalCache, save}
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
/// contains [heel, trim, draught, volume, x, y, z, area, x, y, z, waterline_x, waterline_y]
pub struct DisplacementCache {
    dbg: Dbg,
    cache_path: PathBuf,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    /// Draught in meters
    draught_min: f64,
    draught_max: f64,
    /// qnt draught steps for hull
    draught_step: f64,
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<DisplacementShape>>,
    /// Cache read from `self.file_path`.
    cache: Arc<RwLock<Option<Cache<f64>>>>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl DisplacementCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        cache_dir: impl AsRef<Path>,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_min: f64,
        draught_max: f64,
        draught_step: f64,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, "DisplacementCache");
        let path = cache_dir.as_ref().join("displacement_cache");
        Self {
            shape,
            heel_steps,
            trim_steps,
            draught_min,
            draught_max,
            draught_step,
            cache: Arc::new(RwLock::new(None)),
            cache_path: path,
            dbg,
            scheduler,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Return (draught, center of volume)
    pub fn get(&self, heel: f64, trim: f64, volume: f64, epsilon: f64) -> Result<(f64, Position), Error> {
        let error = Error::new(self.dbg(), "get");     
        let mut step = (self.draught_max - self.draught_min)/2.;
        let mut draught = self.draught_min + step;
        self.init().map_err(|err| error.pass_with("self.init()", err))?;
        let guard = self.cache().read();  
        let cache = guard.as_ref().ok_or(error.pass("no cache"))?;
        for _i in 0..100 {
            let query = [heel, trim, draught];
            let result = cache.get(&query);
            let delta = volume - result.first().ok_or(error.pass("no result from cache.get(&query)"))?;
            if delta.abs() > 30000. {
                panic!("delta.abs() > 30000.");
            }
            if delta.abs() <= epsilon {
            //    dbg!(&query, &result, delta);
                return Ok((draught, Position::new(result[1], result[2], result[3])));
            }
          //  println!("displacement_cache cache get: {_i} {step} {delta} {heel} {trim} {draught} {:?}",  result);
            step = step/2.;
            draught += step*delta.signum();
        }
        Err(error.pass("no result"))
    }
}
//
//
impl LocalCache for DisplacementCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let cache_data = super::build_cache::BuildDisplacementCache::new(
            &self.dbg,
            self.shape.clone(),
            self.heel_steps.clone(),
            self.trim_steps.clone(),
            self.draught_min,
            self.draught_max,
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
