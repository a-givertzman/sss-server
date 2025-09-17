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
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
///
/// Pre-calculated cache for bounds
pub struct BoundDisplacementCache {
    dbg: Dbg,
    cache_path: PathBuf,
    draught_step: f64,
    length_lbp: f64,
    midel_x: f64,
    bounds: Bounds,
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
impl BoundDisplacementCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        cache_dir: PathBuf,
        draught_step: f64,
        length_lbp: f64,
        midel_x: f64,
        bounds: Bounds,
        scheduler: Scheduler,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("BoundDisplacementCache"));
        let cache_path = cache_dir.join(format!("{}", bounds.len_qnt()));
        Self {
            shape,
            draught_step,
            length_lbp,
            midel_x,
            bounds,
            caches: Vec::new(),
            cache_path,
            dbg,
            scheduler,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Return volume in bounds
    /// cause panic if caches not initialized
    pub fn get(&self, draught_mid: f64, trim: f64) -> Vec<f64> {
        let delta_draught = trim.to_radians().sin()*self.length_lbp;
        let result = self
            .caches
            .iter()
            .map(|(dx, cache)| match cache {
                Some(cache) => {
                    let draught =
                        draught_mid + delta_draught * (dx - self.length_lbp / 2. + self.midel_x);
                    cache.get(&vec![draught])[0]
                }
                None => 0.,
            })
            .collect();
        result
    }
    /// Rebuilds a cache
    /// - takes new model
    /// - do calculations
    /// - stores calculated table
    /// - loads recalculated table
    pub fn rebuild(&mut self) -> Result<(), Error> {
    //    let error = Error::new(self.dbg.clone(), "rebuild");
        self.clear_exit();
        match self.calculate() {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::new(self.dbg.clone(), "rebuild").pass(err.to_owned())),
        }
    }
    /// инициализация кэшей заранее посчитанными данными
    fn init(&mut self) -> Result<(), Error> {
        let error = Error::new(self.dbg.clone(), "init");
        self.caches = Vec::new();
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
            self.caches.push((center, cache));
        }
        Ok(())
    }
    //
    fn calculate(&mut self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "calculate");
        let data = super::build_cache::BuildBoundDisplacementCache::new(
            &self.dbg,
            self.shape.clone(),
            self.draught_step,
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
                    Ok(()) => match save(&self.dbg, &self.cache_path.clone().join(format!("{i}")), v) {
                        Ok(()) => (),
                        Err(err) => {
                            let error = error.pass_with("save cache", err);
                            log::error!("{}", error);
                            return Err(error);
                        }
                    },
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
        self.caches = caches;
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
