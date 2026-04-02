use crate::{
    algorithm::entities::{
        Position,
        cache::Cache,
        model_cached::{
            DisplacementCacheResult, DisplacementShape, get_from_volume, local_cache::LocalCache,
            save,
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
        let (draught, result) = get_from_volume(
            &self.dbg,
            cache,
            &[heel, trim],
            volume,
            None,
            None,
            3,
            epsilon,
        )
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
//
#[cfg(test)]
impl DisplacementCache {
    /// Создает "фейковый" кэш гидростатики для тестов.
    /// Позволяет задать предвычисленные значения без прогона тяжелых циклов.
    pub fn create_test_fake(
        mock_cache_data: Option<Cache<f64>>,
    ) -> Self {
        Self {
            dbg: sal_core::dbg::Dbg::new("test", "FakeDisplacementCache"),
            cache_path: PathBuf::from("/tmp/test_displacement_cache"),
            heel_steps: Vec::new(),
            trim_steps: Vec::new(),
            draught_min: 1.,
            draught_max: 10.,
            draught_step: 1.,          
            shape: Arc::new(RwLock::new(unsafe { std::mem::zeroed() })),
            cache: mock_cache_data,
            thread_pool: Arc::new(unsafe { std::mem::zeroed() }),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Вспомогательный метод для создания мока с уже заполненными 
    /// базовыми данными для одной точки (0 крена, 0 дифферента).
    pub fn create_simple_mock(volume: f64, area_wl: f64) -> Self {
        let dbg = sal_core::dbg::Dbg::new("test", "SimpleMockCache");
        let mut cache = Cache::<f64>::new(&dbg);        
        let fake_data = vec![
            (
                vec![
                    0.0, 
                    0.0, 
                    1.0,
                    volume,  // result[0]
                    0.0, 
                    0.0, 
                    -0.5, // result[1,2,3] - Центр величины (x, y, z)
                    area_wl, // result[4]
                    0.0, 0.0, 0.0,  // result[5,6,7] - Центр ВЛ
                    100.0, 500.0,   // result[8,9] - Инерция x, y
                    10.0, 2.0       // result[10,11] - Длина и ширина ВЛ
                ]
            )
        ];        
        let _ = cache.init(fake_data);
        Self::create_test_fake(
            Some(cache)
        )
    }
}