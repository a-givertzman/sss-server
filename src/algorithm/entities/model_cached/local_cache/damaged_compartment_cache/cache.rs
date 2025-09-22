use crate::{
    algorithm::entities::{
        Position,
        cache::Cache,
        model_cached::{DisplacementShape, local_cache::LocalCache, save},
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
pub struct DamagedCompartmentCache {
    dbg: Dbg,
    cache_path: PathBuf,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_min: f64,
    draught_max: f64,
    draught_step: f64,
    ///
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<DisplacementShape>>,
    ///
    /// Cache read from `self.file_path`.
    cache: Option<Cache<f64>>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl DamagedCompartmentCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        cache_dir: impl AsRef<Path>,
        compartment_id: String,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_min: f64,
        draught_max: f64,
        draught_step: f64,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("DamagedCompartment_{compartment_id}_Cache"));
        Self {
            shape,
            heel_steps,
            trim_steps,
            draught_min,
            draught_max,
            draught_step,
            cache: None,
            cache_path: cache_dir.as_ref().join(compartment_id),
            dbg,
            scheduler,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Return (volume, center of volume)
    pub fn get(&self, heel: f64, trim: f64, draught: f64) -> Result<(f64, Position), Error> {
        let error = Error::new(self.dbg(), "get");
        let query = [heel, trim, draught];
        let result = LocalCache::get(self, &query)
            .map_err(|err| error.pass_with(" LocalCache::get(self, &query)", err))?;
        Ok((result[0], Position::new(result[1], result[2], result[3])))
    }
}
//
//
impl LocalCache for DamagedCompartmentCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let cache_data = super::build_cache::BuildDamagedCompartmentCache::new(
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
        let cache = if let Some(cache) = self.cache.take() {
            cache
        } else {
            Cache::<f64>::new(&self.dbg)
        };
        if let Err(err) = cache.init(data.clone()) {
            errors.push(error.pass_with("self.cache.get_mut", err));
        }
        self.cache = Some(cache);
        if let Err(err) = save(&self.dbg, &self.cache_path, data) {
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
        self.cache_path.clone()
    }
    //
    fn cache(&self) -> Option<&Cache<f64>> {
        self.cache.as_ref()
    }
    
    fn set_cache(&mut self, cache: Cache<f64>) {
        self.cache = Some(cache);
    }
}
