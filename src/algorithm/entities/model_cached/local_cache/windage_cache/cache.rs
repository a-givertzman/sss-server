use crate::{
    algorithm::entities::{
        cache::Cache,
        model_cached::{AreaResult, local_cache::LocalCache, save},
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
///
/// Pre-calculated cache for floating position algorithm.
/// contains keys: [heel, trim, draught]
/// values:[volume, x, y, z, area, x, y, z, waterline_x, waterline_y]
pub struct AreaCache {
    dbg: Dbg,
    draught_min: f64,
    cache_path: PathBuf,
    voxels: Option<Vec<(f64, Vec<(f64, f64)>)>>,
    cache: Option<Cache<f64>>,
    thread_pool: Arc<ThreadPool>,
    exit: Arc<AtomicBool>,
}
//
//
impl AreaCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        draught_min: f64,
        voxels: Option<Vec<(f64, Vec<(f64, f64)>)>>,
        cache_dir: impl AsRef<Path>,
        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        let dbg = Dbg::new(parent, "AreaCache");
        let path = cache_dir.as_ref().join("area_cache");
        Self {
            dbg,
            draught_min,
            cache_path: path,
            voxels,
            cache: None,
            thread_pool,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Получение данных кэша для текущего положения
    /// Итерационно подбирает значение водоизмещения по осадке
    /// Паникует если draught выходит за диапазон осадок
    pub fn get(&self, draught: f64) -> Result<AreaResult, Error> {
        assert!(draught > 0.);
        let error = Error::new(self.dbg(), "get");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let query = [draught];
        let result = cache.get(&query); // moment_x, moment_z, area, area_volume_z
        assert!(result.len() == 4);
        let query = [self.draught_min];
        let result_min = cache.get(&query);
        let av_cs_dmin = result_min[2];
        let mv_x_cs_dmin = result_min[0];
        let mv_z_cs_dmin = result_min[1];
        Ok(AreaResult {
            av_cs_dmin,
            mv_x_cs_dmin,
            mv_z_cs_dmin,
            delta_av: av_cs_dmin - result[2],
            delta_mv_x: mv_x_cs_dmin - result[0],
            delta_mv_z: mv_z_cs_dmin - result[1],
            area_volume_z: result[3],
        })
    }
    /// Получение данных кэша для минимальной осадки
    /// Итерационно подбирает значение водоизмещения по осадке
    /// Паникует если draught выходит за диапазон осадок
    pub fn get_min(&self) -> Result<(f64, f64, f64), Error> {
        let error = Error::new(self.dbg(), "get_min");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let query = [self.draught_min];
        let result_min = cache.get(&query);
        let _av_cs_dmin = result_min[2];
        let _mv_x_cs_dmin = result_min[0];
        let _mv_z_cs_dmin = result_min[1];
        Ok((
            result_min[2],
            result_min[0],
            result_min[1],
        ))
    }
}
//
//
impl LocalCache for AreaCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        //   dbg!("AreaCache calculate begin");
        let error = Error::new(&self.dbg, "calculate");
        let voxels = match self.voxels.as_ref() {
            Some(voxels) => voxels,
            None => {
                let error = error.pass("no voxels");
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
        let data: Vec<(f64, f64, f64)> = voxels
            .iter()
            .flat_map(|(x, v)| v.iter().map(|(z, a)| (*x, *z, *a)).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let data = Arc::new(data);
        let mut tasks: VecDeque<JoinHandle<_>> = VecDeque::new();
        let results = Arc::new(Stack::new());
        let scheduler = self.thread_pool.scheduler();
        'draught: for draught in draught_array {
            if self.exit.load(Ordering::SeqCst) {
                break 'draught;
            }
            //  let dbg_ = self.dbg.clone();
            let results = results.clone();
            let data: Arc<Vec<_>> = Arc::clone(&data);
            let thread_name = format!("AreaCache calculate {draught}");
            log::info!("{}.build | Starting thread {thread_name}", &self.dbg);
            //  println!("Starting thread {thread_name}");
            let handle = scheduler
                .spawn_named(thread_name, move || {
                    let (windage, volume): (Vec<_>, Vec<_>) =
                        data.iter().partition(|(_, z, _)| *z > draught);
                    let (moment_x, moment_z, area) = windage
                        .into_iter()
                        .fold((0., 0., 0.), |(sum_x, sum_z, sum_a), (x, z, a)| {
                            (sum_x + x * a, sum_z + z * a, sum_a + a)
                        });
                    let (area_volume, volume_moment) = volume
                        .into_iter()
                        .fold((0., 0.), |(sum_a, sum_m), (_, z, a)| {
                            (sum_a + a, sum_m + z * a)
                        });
                    let area_volume_z = if area_volume > 0. {
                        volume_moment / area_volume
                    } else {
                        0.
                    };
                    results.push((draught, moment_x, moment_z, area, area_volume_z));
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
        for task in tasks {
            log::trace!("join thread {}", task.name());
            if let Err(err) = task.join() {
                pass("task join", err);
            }
        }
        let mut vec_results = Vec::new();
        while !results.is_empty() {
            if let Some((draught, moment_x, moment_z, area, area_volume_z)) = results.pop() {
                vec_results.push(vec![draught, moment_x, moment_z, area, area_volume_z]);
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
        //    dbg!("AreaCache calculate end");
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
