use crate::{
    algorithm::entities::{
        Position,
        cache::Cache,
        model_cached::{
            DisplacementCacheResult, DisplacementShape, get_from_volume, local_cache::LocalCache, save,
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
/// contains keys: [heel, trim, draught]
/// values:[volume, x, y, z, area, x, y, z, waterline_x, waterline_y]
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
    cache: Option<Cache<f64>>,
    thread_pool: Arc<ThreadPool>,
    exit: Arc<AtomicBool>,
}
//
//
impl DisplacementCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    /// TODO - panic
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        cache_dir: impl AsRef<Path>,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_min: f64,
        draught_max: f64,
        draught_step: f64,
        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        let dbg = Dbg::new(parent, "DisplacementCache");
        let path = cache_dir.as_ref().join("displacement_cache");
        assert!(!heel_steps.is_empty());
        assert!(!trim_steps.is_empty());
        Self {
            shape,
            draught_min,
            draught_max,
            heel_steps,
            trim_steps,
            draught_step,
            cache: None,
            cache_path: path,
            dbg,
            thread_pool,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Получение данных кэша для текущего положения
    /// Итерационно подбирает значение водоизмещения по осадке    
    pub fn get(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
    ) -> Result<DisplacementCacheResult, Error> {
        let error = Error::new(self.dbg(), "get");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let (draught, result) = get_from_volume(&self.dbg, cache, &[heel, trim], volume, 3, epsilon)
            .map_err(|err| error.pass(err))?;
        Ok(DisplacementCacheResult {
            heel,
            trim,
            draught,
            volume: result[0],
            volume_center: Position::new(result[1], result[2], result[3]),
            area_wl: result[4],
            area_wl_center: Position::new(result[5], result[6], result[7]),
            inertia_trans_x: result[8],
            inertia_long_y: result[9],
            length_wl: result[10],
            breadth_wl: result[11],
        })
    } 
    //
    pub fn get_volume_disp(&self) -> Result<(f64, f64), Error> {
        let error = Error::new(self.dbg(), "get_max_volume");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        Ok(cache.disp(3))
    }
}
//
impl LocalCache for DisplacementCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        //   dbg!("DisplacementCache calculate begin");
        let error = Error::new(&self.dbg, "calculate");
        let (data, mut errors) = super::build_cache::BuildDisplacementCache::new(
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
        self.set_cache(cache);
        if let Err(err) = save(&self.dbg, &self.cache_path, data) {
            errors.push(error.pass_with("save data", err));
        }
        //    dbg!("DisplacementCache calculate end");
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
        let _ = self.cache.insert(cache);
    }
}
