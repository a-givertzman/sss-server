use crate::{
    algorithm::entities::{
        Position,
        cache::Cache,
        model_cached::{DisplacementShape, get_from_level, local_cache::LocalCache, save},
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
pub struct DamagedCompartmentCache {
    dbg: Dbg,
    cache_path: PathBuf,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    /// коэффициент проницаемости
    coeff: Option<f64>,    
    draught_min: f64,
    draught_max: f64,
    draught_step: f64,
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<DisplacementShape>>,
    /// Cache read from `self.file_path`.
    cache: Option<Cache<f64>>,
    thread_pool: Arc<ThreadPool>,
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
        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("DamagedCompartmentCache_{compartment_id}"));
        Self {
            shape,
            heel_steps,
            trim_steps,
            coeff: None,            
            draught_min,
            draught_max,
            draught_step,
            cache: None,
            cache_path: cache_dir.as_ref().join(compartment_id),
            dbg,
            thread_pool,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Расчет [коэффициента проницаемости](https://github.com/a-givertzman/sss/blob/master/design/algorithm-simply/part02_mass/chapter04_volumeNetto.md)
    pub fn calc_coeff(&mut self, volume_max: Option<f64>) -> Result<(), Error> {
        let error = Error::new(self.dbg(), "calc_coeff");
        let volume_brutto = self.cache.as_ref().ok_or(error.pass("no cache"))?.disp(3).1;
        self.coeff = Some(if volume_brutto > 0. {
            volume_max.unwrap_or(volume_brutto) / volume_brutto
        } else {
            1.
        });
        // println!("compartment_cache calc_coeff {} {:.3} {:.3} {:.3}", self.dbg(), volume_max, volume_brutto, self.coeff.unwrap());
        Ok(())
    }  
    /// Return (volume, center of volume)
    pub fn get(&self, heel: f64, trim: f64, draught: f64) -> Result<(f64, Position), Error> {
        let error = Error::new(self.dbg(), "get");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let query = [heel, trim];
        let result = get_from_level(&self.dbg, cache, &query, draught, None, 3)
            .map_err(|err| error.pass_with("get_from_level", err))?;
        let coeff = self.coeff.as_ref().ok_or(error.pass("no coeff"))?; 
        Ok((result[0] * coeff, Position::new(result[1], result[2], result[3])))
    }
}
//
//
impl LocalCache for DamagedCompartmentCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let (data, mut errors) = super::build_cache::BuildDamagedCompartmentCache::new(
            &self.dbg,
            self.shape.clone(),
            self.heel_steps.clone(),
            self.trim_steps.clone(),
            self.draught_min,
            self.draught_max,
            self.draught_step,
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
