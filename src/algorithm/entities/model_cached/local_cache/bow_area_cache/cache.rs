use crate::{
    algorithm::entities::{
        cache::Cache,
        model_cached::{local_cache::LocalCache, save},
    },
    kernel::types::Arc,
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, ThreadPool},
};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
/// Pre-calculated cache for bow area cache
/// contains keys: [draught]
/// values:[area]
pub struct BowAreaCache {
    dbg: Dbg,
    cache_path: PathBuf,
    voxels: Option<Vec<(f64, Vec<(f64, f64)>)>>,
    voxel_scale: Option<f64>,
    cache: Option<Cache<f64>>,
    thread_pool: Arc<ThreadPool>,
    exit: Arc<AtomicBool>,
}
//
//
impl BowAreaCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        voxels: Option<Vec<(f64, Vec<(f64, f64)>)>>,
        voxel_scale: Option<f64>,        
        cache_dir: impl AsRef<Path>,
        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        let dbg = Dbg::new(parent, "BowAreaCache");
        let path = cache_dir.as_ref().join("bow_area_cache");
        Self {
            dbg,
            voxels,
            voxel_scale,
            cache_path: path,
            cache: None,
            thread_pool,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Получение данных кэша для текущего положения
    /// Итерационно подбирает значение водоизмещения по осадке
    /// Паникует если draught выходит за диапазон осадок
    pub fn get(&self, trim: f64, draught: f64) -> Result<f64, Error> {
        assert!(draught > 0.);
        let error = Error::new(self.dbg(), "get");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let query = [trim, draught];
        let result = cache.get(&query);
        assert!(result.len() == 1);
        Ok(result[0])
    }
}
//
//
impl LocalCache for BowAreaCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        //   dbg!("BowAreaCache calculate begin");
        let error = Error::new(&self.dbg, "calculate");
        let voxels = match self.voxels.as_ref() {
            Some(voxels) => voxels.clone(),
            None => {
                let error = error.pass("no voxels");
                log::error!("{:?}", &error);
                return vec![error];
            }
        };
        let voxel_scale = match self.voxel_scale {
            Some(voxel_scale) => voxel_scale,
            None => {
                let error = error.pass("no voxel_scale");
                log::error!("{:?}", &error);
                return vec![error];
            }
        };
        let mut errors = Vec::new();
        let mut pass = |message: &str, err: Error| {
            let error = error.pass_with(message, err);
            log::error!("{:?}", &error);
            errors.push(error);
        };
        let mut draught_array: Vec<f64> = voxels
            .iter()
            .flat_map(|(_, v)| v.iter().map(|(z, _)| *z).collect::<Vec<f64>>())
            .collect();
        draught_array.push(0.);
        draught_array.sort_by(|a, b| a.partial_cmp(b).unwrap());
        draught_array.dedup();
        let trim_array: Vec<_> = (-40..=40).map(|v| v as f64).collect();
        let voxels = Arc::new(voxels);
        let mut tasks: VecDeque<JoinHandle<_>> = VecDeque::new();
        let results = Arc::new(Stack::new());
        let scheduler = self.thread_pool.scheduler();
        'draught: for draught in draught_array {
            for &trim in &trim_array {
                if self.exit.load(Ordering::SeqCst) {
                    break 'draught;
                }
                //  let dbg_ = self.dbg.clone();
                let results = results.clone();
                let thread_name = format!("BowAreaCache calculate {trim} {draught}");
                let voxels = Arc::clone(&voxels);
                log::info!("{}.build | Starting thread {thread_name}", &self.dbg);
             //   println!("Starting thread {thread_name}");
                let handle = scheduler
                    .spawn_named(thread_name, move || {
                        let area = bow_area(voxels, voxel_scale, trim, draught);
                        results.push((trim, draught, area));
                        Ok(())
                    })
                    .map_err(|err| {
                        error.pass_with(format!("spawn task draught:{draught}"), err.to_string())
                    });
                match handle {
                    Ok(task) => tasks.push_back(task),
                    Err(err) => pass("task handle", err),
                };
            }
        }
        for task in tasks {
            log::trace!("join thread {}", task.name());
            if let Err(err) = task.join() {
                pass("task join", err);
            }
        }
        let mut vec_results = Vec::new();
        while !results.is_empty() {
            if let Some((trim, draught, area)) = results.pop() {
                vec_results.push(vec![trim, draught, area]);
            }
        }
        let cache = if let Some(cache) = self.cache.take() {
            cache
        } else {
            Cache::<f64>::new(&self.dbg)
        };
        if let Err(err) = cache.init(vec_results.clone()) {
            errors.push(error.pass_with("self.cache.get_mut", err));
        }
        self.set_cache(cache);
        if let Err(err) = save(&self.dbg, &self.cache_path, vec_results) {
            errors.push(error.pass_with("save data", err));
        }
        //    dbg!("BowAreaCache calculate end");
        errors
    }
    //
    fn exit(&self) {
        
    }
    //
    fn clear_exit(&self) {
        
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

/// Расчет площади проекции по правилу дополнительного запаса плавучести в носу
/// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part03_draft/chapter02_draftCriteria/section04_bowBuoyancy.md]
/// Возвращает повернутое и смещенное разбиение [dx, area]
fn bow_area(voxels: Arc::<Vec<(f64, Vec<(f64, f64)>)>>, voxel_scale: f64, trim: f64, draught: f64) -> f64 {
    let sin_trim = trim.to_radians().sin();
    voxels
        .iter()
        .map(|(x, v)| {
            let x = *x;
            let draught = draught + x * sin_trim;
            let draught_l = draught - voxel_scale / 2.;
            let draught_h = draught + voxel_scale / 2.;
            v
                .iter()
                .filter(|&&(z, _)| z > draught_l)
                .map(|&(z, a)|
                    if z < draught_h {
                        (draught_h - z)*a / voxel_scale
                    } else {
                        a
                    }
                )
                .sum::<f64>()
        })
        .sum()
}
