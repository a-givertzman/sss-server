use crate::{
    algorithm::entities::{
        Bounds,
        cache::Cache,
        model_cached::{DisplacementShape, read, save},
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::Scheduler;
use serde_json::error;
use std::{
    path::PathBuf,
    sync::{OnceLock, atomic::{AtomicBool, Ordering}},
};
///
/// Pre-calculated cache for bounds
pub struct BoundDisplacementCache {
    dbg: Dbg,
    cache_path: PathBuf,
    level_step: f64,
    bounds: Bounds,
    ///
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<DisplacementShape>>,
    ///
    /// Cache read from `self.file_path`.
    caches: OnceLock<Vec<(f64, Option<Cache<f64>>)>>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl BoundDisplacementCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        cache_dir: PathBuf,
        level_step: f64,
        bounds: Bounds,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("BoundDisplacementCache"));
        let cache_path = cache_dir.join(format!("{}", bounds.len_qnt()));
        Self {
            shape,
            level_step,
            bounds,
            caches: OnceLock::new(),
            cache_path,
            dbg,
            scheduler,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Return volume in bounds
    /// cause panic if caches not initialized
    pub fn get(&self, draught_mid: f64, trim: f64) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "get"); 
        let caches = self.caches.get().ok_or(error.pass("no caches"))?;
        //    let delta_draught = trim.to_radians().sin()*self.length_lbp;
        let result = caches
            .iter()
            .map(|(dx, cache)| match cache {
                Some(cache) => {
                    //            let draught = draught_mid + delta_draught * (dx - self.length_lbp / 2. + self.midel_x);
                    let draught = draught_mid + dx * trim.to_radians().sin();
                    dbg!(draught_mid, dx, draught);
                    cache.get(&vec![draught])[0]
                }
                None => 0.,
            })
            .collect();
        Ok(result)
    }
    /// Return max volume in bounds
    /// cause panic if caches not initialized
    pub fn get_max(&self) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "get_max"); 
        let caches = self.caches.get().ok_or(error.pass("no caches"))?;
        let result = caches
            .iter()
            .map(|(_, cache)| match cache {
                Some(cache) => cache.max_value(1),
                None => 0.,
            })
            .collect();
        Ok(result)
    }
    /// Rebuilds a cache
    /// - takes new model
    /// - do calculations
    /// - stores calculated table
    /// - loads recalculated table
    pub fn rebuild(&mut self) -> Result<(), Error> {
        //    let error = Error::new(self.dbg.clone(), "rebuild");
        dbg!("rebuild");
        self.clear_exit();
        match self.calculate() {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::new(self.dbg.clone(), "rebuild").pass(err.to_owned())),
        }
    }
    /// инициализация кэшей заранее посчитанными данными
    pub fn init(&self) -> Result<(), Error> {
    //    dbg!(self.dbg.clone(), "init", &self.cache_path.clone());
        let error = Error::new(self.dbg.clone(), "init");
        let mut caches = Vec::new();
        for (i, bound) in self.bounds.iter().enumerate() {
            let cache =
                if let Ok(vals) = read(&self.dbg, &self.cache_path.clone().join(format!("{i}"))) {
                    let cache = Cache::new(&self.dbg);
                    cache
                        .init(vals)
                        .map_err(|err| error.pass_with("cache.init error", err))?;
                    Some(cache)
                } else {
                    None
                };
            let center = bound.center().ok_or(error.err("bound.center()"))?;
            caches.push((center, cache));
        }
        self.caches.set(caches).map_err(|_| error.err("caches.set"))?;
        Ok(())
    }
    //
    fn calculate(&mut self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "calculate");
        let data = super::build_cache::BuildBoundDisplacementCache::new(
            &self.dbg,
            self.shape.clone(),
            self.level_step,
            self.bounds.clone(),
            self.scheduler.clone(),
            self.exit.clone(),
        )
        .build();
        let data = match data {
            Ok(data) => data,
            Err(err) => return Err(error.pass_with("cache_data", err)),
        };
        let mut caches = Vec::new();
        for (i, (dx, v)) in data.into_iter().enumerate() {
            if self.exit.load(Ordering::Relaxed) {
                return Err(error.err("exit"));
            }
            let cache = if let Some(v) = v {
                let v: Vec<Vec<f64>> = v.iter().map(|v| vec![v.0, v.1]).collect();
                let cache = Cache::<f64>::new(&self.dbg);
                match cache.init(v.clone()) {
                    Ok(()) => {
                        match save(&self.dbg, &self.cache_path.clone().join(format!("{i}")), v) {
                            Ok(()) => (),
                            Err(err) => {
                                let error = error.pass_with("save cache", err);
                                log::error!("{}", error);
                                return Err(error);
                            }
                        }
                    }
                    Err(err) => {
                        let error = error.pass_with("cache.init()", err);
                        log::error!("{}", error);
                        return Err(error);
                    }
                }
                Some(cache)
            } else {
                None
            };
            caches.push((dx, cache));
        }
        self.caches.set(caches).map_err(|_| error.err("caches.set"))?;
        Ok(())
    }
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst)
    }
    //
    fn clear_exit(&self) {
        self.exit.store(false, Ordering::SeqCst)
    }
}
