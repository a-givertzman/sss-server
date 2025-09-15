use crate::{
    algorithm::entities::{
        Bounds, cache::Cache, model_cached::{DisplacementShape, save}
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::Scheduler;
use std::{path:: PathBuf, sync::atomic::{AtomicBool, Ordering}};
///
/// Pre-calculated cache for bounds
pub struct BoundCache {
    dbg: Dbg,
    cache_dir: PathBuf,
    draught_min: f64,
    draught_max: f64,
    draught_step: f64,
    length_lbp: f64,
    midel_x: f64,
    ///
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<DisplacementShape>>,
    ///
    /// Cache read from `self.file_path`.
    caches: Vec<(f64, Option<Cache<f64>>)>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl BoundCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        cache_dir: PathBuf,
        draught_min: f64,
        draught_max: f64,
        draught_step: f64,
        length_lbp: f64,
        midel_x: f64,
        delta_x: Vec<f64>,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("BoundCache"));
        Self {
            shape,
            draught_min,
            draught_max,
            draught_step,
            length_lbp,
            midel_x,
            caches: delta_x.into_iter().map(|dx| (dx, None)).collect(),
            cache_dir,
            dbg,
            scheduler,
            exit: Arc::new(AtomicBool::new(false)),
        }        
    }
    /// Return volume in bounds
    /// cause panic if caches not initialized
    pub fn get(&self, draught_mid: f64, trim_m: f64) -> Vec<f64> {
        let delta_draught = trim_m/self.length_lbp;
        let result = self.caches
            .iter()
            .map(|(dx, cache)| {
                let draught = draught_mid
                    + delta_draught
                        * (dx - self.length_lbp / 2. + self.midel_x);
                    cache.as_ref().unwrap().get(&vec![draught])[0]
            })
            .collect();
        result
    }
    /// Rebuilds a cache
    /// - takes new model
    /// - do calculations
    /// - stores calculated table
    /// - loads recalculated table
    fn rebuild(&mut self) -> Result<(), Error> {
        self.clear_exit();
        match self.calculate().first() {
            Some(err) => Err(Error::new(self.dbg(), "rebuild").pass(err.to_owned())),
            None => Ok(()),
        }
    }
    /// инициализация кэша заранее посчитанными данными
    fn init(&self) -> Result<(), Error> {
        let error = Error::new(self.dbg(), "init");
        if self.cache().read().is_none() {
            let vals = read(self.dbg(), self.cache_path())
                .map_err(|err| error.pass_with("read cache data error", err))?;
            let cache = Cache::new(self.dbg());            
            cache
                .init(vals)
                .map_err(|err| error.pass_with("cache.init error", err))?;
            let _ = self.cache().write().insert(cache);
        }
        Ok(())
    }
    //
    fn calculate(&mut self, bounds: Bounds) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let cache_data = super::build_cache::BuildBoundCache::new(
            &self.dbg,
            self.shape.clone(),
            self.draught_min - self.length_lbp,
            self.draught_max + self.length_lbp,
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
