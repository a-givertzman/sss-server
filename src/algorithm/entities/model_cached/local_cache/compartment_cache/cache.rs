use crate::{
    algorithm::entities::{
        Bounds, Position,
        cache::Cache,
        model_cached::{
            CompartmentBoundCache, CompartmentCacheResult, DisplacementShape,
            local_cache::LocalCache, save,
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
pub struct CompartmentCache {
    dbg: Dbg,
    cache_dir: PathBuf,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    level_step_qnt: usize,
    /// Максимальный объем отсека из БД (Нетто)
    volume_max: Option<f64>,
    /// коэффициент проницаемости
    coeff: Option<f64>,
    /// Model representation used for cache calculation.
    shape: Arc<RwLock<DisplacementShape>>,
    /// Cache read from `self.file_path`.
    cache: Option<Cache<f64>>,
    thread_pool: Arc<ThreadPool>,
    exit: Arc<AtomicBool>,
}
//
//
impl CompartmentCache {
    ///
    /// Creates a new instance.
    /// * cache_dir - folder contains all cache files
    /// * volume_max - полный объем из бд
    pub fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        cache_dir: impl AsRef<Path>,
        compartment_id: String,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        level_step_qnt: usize,
        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("CompartmentCache_{compartment_id}"));
        Self {
            shape,
            heel_steps,
            trim_steps,
            level_step_qnt,
            volume_max: None,
            coeff: None,
            cache: None,
            cache_dir: cache_dir.as_ref().join(compartment_id),
            dbg,
            thread_pool,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Расчет коэффициента проницаемости
    pub fn calc_coeff(&mut self, volume_max: f64) -> Result<(), Error> {
        let error = Error::new(self.dbg(), "calc_coeff");
        let volume_brutto = self
            .cache
            .as_ref()
            .ok_or(error.pass("no cache"))?
            .value_disp(3)
            .1;
        self.volume_max = Some(volume_max);
        self.coeff = Some(if volume_brutto > 0. {
            volume_max / volume_brutto
        } else {
            1.
        });
    //    println!("compartment_cache calc_coeff {} {:.3} {:.3} {:.3}", self.dbg(), volume_max, volume_brutto, self.coeff.unwrap());
        Ok(())
    }
    /// Получение значения для заданных условий для расчета дсо
    /// объем изменяется исходя из признаков
    pub fn get_for_dso(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
        use_max_moment: bool, //признак пересчета объема от макс. момента
        is_cargo_tank: bool, 
    ) -> Result<CompartmentCacheResult, Error> {
        let error = Error::new(self.dbg(), "get");
    //    println!("compartment_cashe {} get_for_dso start:  heel:{heel} trim:{trim} volume:{volume} epsilon:{epsilon} use_max_moment:{use_max_moment} is_cargo_tank:{is_cargo_tank}", self.dbg);
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let mut result = self.get_for_floating(
            heel,
            trim,
            volume,
            epsilon,
        ).map_err(|err| error.pass(err))?;
        if !is_cargo_tank {// Для всех цистерн кроме грузовых
            if use_max_moment {
                result.volume = result.volume_from_moment;
                result.volume_center = result.volume_from_moment_center;
            } else {
                let volume_max = cache.value_disp(3).1;// максимальный объем
                if volume >= volume_max*0.98 {
                    result.inertia_trans_x = 0.;
                    result.volume = 0.;
                }
            }
        } else {
            // Для грузовых цистерн (перевозимых полезный груз, "CompartmentPurpose"="cargo_tank)
            // считаем при крене 5 градусов
            let heel = 5.0*heel.signum();
            let CompartmentCacheResult{inertia_trans_x, ..} = self.get_for_floating(
                heel,
                trim,
                volume,
                epsilon,
            ).map_err(|err| error.pass(err))?;
            result.inertia_trans_x = inertia_trans_x;
        }
    //    println!("compartment_cashe {} get_for_dso ok: heel:{heel} volume:{volume} result.volume:{} y:{}", self.dbg, result.volume, result.volume_center.y());
        return Ok(result);
    }
    /// Получение значения из кэша для заданных условий для расчета равнвесного положения
    /// https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter01_initialStability/chapter01_initialStability.md\
    pub fn get_for_floating(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
    ) -> Result<CompartmentCacheResult, Error> {
        let error = Error::new(self.dbg(), "_get");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let coeff = self.coeff.as_ref().ok_or(error.pass("no coeff"))?;
        let volume_ = volume / coeff;
        let level_max = cache.value_disp(2).1;
        let mut level = level_max / 2.;
        let mut step = level_max / 4.;
        let mut last_delta_signum = 1.;       
        for i in 0..=50 {
            let query = [heel, 0., level];
            let result = cache.get(&query);
            assert!(result.len() >= 6);
            let delta = result
                .first()
                .ok_or(error.pass("no result from cache.get(&query)"))?
                - volume_;
        //    println!("compartment_cashe {} get_for_floating heel:{heel} level:{level} volume:{} coeff:{coeff} volume_:{volume_} delta:{delta} y:{}", self.dbg, result[0], result[2]);
            if delta.abs() <= epsilon || i >= 50 {
        //        println!("compartment_cashe {} get_for_floating heel:{heel} volume:{volume} coeff:{coeff} volume_:{volume_} delta:{delta} y:{}", self.dbg, result[2]);
                return Ok(CompartmentCacheResult {
                    heel,
                    trim,
                    level,
                    volume: volume_ * coeff,
                    volume_center: Position::new(result[1], result[2], result[3]),
                    inertia_trans_x: result[4],
                    inertia_long_y: result[5],
                    moment_max: result[6],
                    volume_from_moment: result[7],
                    volume_from_moment_center: Position::new(result[8], result[9], result[10]),
                });
            }
            if last_delta_signum != delta.signum() {
                step = step * 0.3;
                last_delta_signum = delta.signum();
            }
            level -= step * delta.signum();
        }
        Err(error.pass(format!("no result for epsilon:{epsilon}")))
    }
 /*   /// Получение значения кэша для для максимальной поправки при нулевых крене и дифференте
    /// Возвращает объем и значение поперечного момента
    pub fn _get_with_max_moment(&self, heel: f64) -> Result<(f64, Position, f64), Error> {
        let error = Error::new(self.dbg(), "_get_with_max_moment");
        println!("compartment_cashe {} _get_with_max_moment start:  heel:{heel}", self.dbg);
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let coeff = self.coeff.as_ref().ok_or(error.pass("no coeff"))?;
        let result = cache.value_disp_opt(8, &[Some(heel)])
            .ok_or(error.pass("no result"))?;
        assert!(result.1.len() >= 8);
        println!("compartment_cashe {} _get_with_max_moment ok heel:{heel} result:{:?} ", self.dbg, result);
        Ok((result.1[3]*coeff, Position::new(result.1[4], result.1[5], result.1[6]), result.1[7]))  
    }
*/
/*
   pub fn get(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
        use_max_inertia_trans: bool,
        is_cargo_tank: bool, 
    ) -> Result<CompartmentCacheResult, Error> {
        let error = Error::new(self.dbg(), "get");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let coeff = self.coeff.as_ref().ok_or(error.pass("no coeff"))?;
        let volume = volume / coeff;
        let level_max = cache.value_disp(2).1;
        let mut step = level_max / 2.;
        let mut level = step;
        for i in 0..=50 {
            let query = [heel, trim, level];
            let result = cache.get(&query);
            assert!(result.len() == 6);
            let delta = result
                .first()
                .ok_or(error.pass("no result from cache.get(&query)"))?
                - volume;
            if delta.abs() <= epsilon || i >= 50 {
                return Ok(CompartmentCacheResult {
                    heel,
                    trim,
                    level,
                    volume: volume * coeff,
                    volume_center: Position::new(result[1], result[2], result[3]),
                    inertia_trans_x: result[4],
                    inertia_long_y: result[5],
                });
            }
            step = step / 2.;
            level -= step * delta.signum();
        }
        Err(error.pass(format!("no result for epsilon:{epsilon}")))
    }
*/



    //
    pub fn build_bounded(
        &self,
        bounds: Bounds,
        level_step: f64,
    ) -> Result<CompartmentBoundCache, Error> {
        let volume_max = self
            .volume_max
            .as_ref()
            .ok_or(Error::new(self.dbg(), "build_bounded").err("no volume_max"))?
            .clone();
        Ok(CompartmentBoundCache::new(
            &self.dbg,
            self.shape.clone(),
            volume_max,
            self.cache_dir.clone().join("distr"),
            level_step,
            bounds,
            Arc::clone(&self.thread_pool),
        ))
    }
}
//
//
impl LocalCache for CompartmentCache {
    //
    fn calculate(&mut self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let (data, mut errors) = super::build_cache::BuildCompartmentCache::new(
            &self.dbg,
            self.shape.clone(),
            self.heel_steps.clone(),
            self.trim_steps.clone(),
            self.level_step_qnt,
            Arc::clone(&self.thread_pool),
            self.exit.clone(),
        )
        .build();
        if !errors.is_empty() {
            return errors;
        }
        let cache = if let Some(cache) = self.cache.take() {
            cache
        } else {
            Cache::<f64>::new(&self.dbg)
        };
        if let Err(err) = cache.init(data.clone()) {
            errors.push(error.pass_with("self.cache.get_mut", err));
        }
        self.set_cache(cache);
        if let Err(err) = save(&self.dbg, &self.cache_path(), data) {
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
        self.cache_dir.clone().join("disp")
    }
    //
    fn cache(&self) -> Option<&Cache<f64>> {
        self.cache.as_ref()
    }
    //
    fn set_cache(&mut self, cache: Cache<f64>) {
        let _ = self.cache.insert(cache);
    }
}
