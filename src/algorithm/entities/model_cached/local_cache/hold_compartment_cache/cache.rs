use crate::{
    algorithm::entities::{
        Bounds, Curve, ICurve, Position,
        cache::Cache,
        model_cached::{
            CompartmentBoundCache, CompartmentCacheResult, DisplacementShape, get_volume, local_cache::LocalCache, save
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
pub struct HoldCompartmentCache {
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
impl HoldCompartmentCache {
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
        let volume_brutto = self.cache.as_ref().ok_or(error.pass("no cache"))?.disp(3).1;
        self.volume_max = Some(volume_max);
        self.coeff = Some(if volume_brutto > 0. {
            volume_max / volume_brutto
        } else {
            1.
        });
        // println!("compartment_cache calc_coeff {} {:.3} {:.3} {:.3}", self.dbg(), volume_max, volume_brutto, self.coeff.unwrap());
        Ok(())
    }
    /// Получение значения для заданных условий для расчета дсо
    /// на основе поперечного момента инерции площади ватерлинии.
    /// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter02_bigAngles/deltaL.md]
    /// Oбъем изменяется исходя из признаков
    pub fn get_for_dso_surface_moment(
        &self,
        volume: f64,
        epsilon: f64,
        use_max_moment: bool, //признак пересчета объема от макс. момента
        is_cargo_tank: bool,
    ) -> Result<f64, Error> {
        let error = Error::new(self.dbg(), "get_Ixx");
        //    println!("compartment_cashe {} get_for_dso start:  heel:{heel} trim:{trim} volume:{volume} epsilon:{epsilon} use_max_moment:{use_max_moment} is_cargo_tank:{is_cargo_tank}", self.dbg);
        let current = self
            .get(0., 0., volume, epsilon)
            .map_err(|err| error.pass(err))?;
        if !is_cargo_tank && use_max_moment {
            //        println!("gdfhgfhjyjf volume:{} current:{} balanced:{} delta:{};", max_moment.1, max_moment.2, max_moment.3, max_moment.0);
            return Ok(current.max_inertia_trans_x);
        }
        if !is_cargo_tank {
            let coeff = self.coeff.as_ref().ok_or(error.pass("no coeff"))?;
            let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
            let (_, max_volume) = cache.disp(3);
            let volume = volume / coeff;
            if volume >= max_volume * 0.98 {
                return Ok(0.);
            }
        }
        //      println!("gdfhgfhjyjf volume:{} current:{} balanced:{} delta:{};", volume, current.abs_moment, balanced.abs_moment, res);
        Ok(current.inertia_trans_x)
    }
    /// Получение значения для заданных условий для расчета дсо
    /// на основе фактического кренящего момента.
    /// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter02_bigAngles/deltaL.md]
    /// Объем изменяется исходя из признаков
    pub fn get_for_dso_abs_moment(
        &self,
        current_heel: f64,
        current_trim: f64,
        volume: f64,
        balanced_heel: f64,
        balanced_trim: f64,
        epsilon: f64,
        use_max_moment: bool, //признак пересчета объема от макс. момента
        is_cargo_tank: bool,
    ) -> Result<f64, Error> {
        let error = Error::new(self.dbg(), "get_for_dso");
        //    println!("compartment_cashe {} get_for_dso start:  heel:{heel} trim:{trim} volume:{volume} epsilon:{epsilon} use_max_moment:{use_max_moment} is_cargo_tank:{is_cargo_tank}", self.dbg);
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let coeff = self.coeff.as_ref().ok_or(error.pass("no coeff"))?;
        let (_, max_volume) = cache.disp(3);
        if !is_cargo_tank && use_max_moment {
            let balanced = cache.values_disp(&[Some(balanced_heel), Some(balanced_trim), None]);
            let balanced = Curve::new_linear(
                &balanced
                    .into_iter()
                    .map(|v| (v[0], v[7]))
                    .collect::<Vec<_>>(),
            )
            .map_err(|err| error.pass_with("balanced", err))?;
            let current = cache.values_disp(&[Some(current_heel), Some(current_trim), None]);
            let current = Curve::new_linear(
                &current
                    .into_iter()
                    .map(|v| (v[0], v[7]))
                    .collect::<Vec<_>>(),
            )
            .map_err(|err| error.pass_with("current", err))?;
            let step = max_volume * 0.1 / (self.level_step_qnt as f64);
            let volume_steps: Vec<_> = (0..=self.level_step_qnt * 10)
                .map(|v| (v as f64) * step)
                .collect();
            let mut values = Vec::new();
            for volume in volume_steps {
                let current = current
                    .value(volume)
                    .map_err(|err| error.pass_with("current", err))?;
                let balanced = balanced
                    .value(volume)
                    .map_err(|err| error.pass_with("balanced", err))?;
                values.push((current - balanced, volume, current, balanced));
            }
            let max_moment = if current_heel >= balanced_heel {
                values
                    .iter()
                    .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                    .ok_or(error.pass("max_moment"))?
            } else {
                values
                    .iter()
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                    .ok_or(error.pass("max_moment"))?
            };
            //        println!("gdfhgfhjyjf volume:{} current:{} balanced:{} delta:{};", max_moment.1, max_moment.2, max_moment.3, max_moment.0);
            return Ok(max_moment.0 * coeff);
        }
        if !is_cargo_tank {
            let volume = volume / coeff;
            if volume >= max_volume * 0.98 {
                return Ok(0.);
            }
        }
        let current = self
            .get(current_heel, current_trim, volume, epsilon)
            .map_err(|err| error.pass(err))?;
        let balanced = self
            .get(balanced_heel, balanced_trim, volume, epsilon)
            .map_err(|err| error.pass(err))?;
        let res = current.abs_moment - balanced.abs_moment;
        //      println!("gdfhgfhjyjf volume:{} current:{} balanced:{} delta:{};", volume, current.abs_moment, balanced.abs_moment, res);
        Ok(res)
    }
    /// Получение значения для заданных условий для расчета влияния свободной поверхности
    /// объем изменяется исходя из признаков
    /// https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter01_initialStability/chapter01_initialStability.md
    pub fn get_for_stability(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
        use_max_moment: bool, //признак пересчета объема от макс. момента
        is_cargo_tank: bool,
    ) -> Result<CompartmentCacheResult, Error> {
        let error = Error::new(self.dbg(), "get_for_stability");
        //    println!("compartment_cashe {} get_for_dso start:  heel:{heel} trim:{trim} volume:{volume} epsilon:{epsilon} use_max_moment:{use_max_moment} is_cargo_tank:{is_cargo_tank}", self.dbg);
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let mut result = self
            .get(heel, trim, volume, epsilon)
            .map_err(|err| error.pass(err))?;
        if !is_cargo_tank {
            // Для всех цистерн кроме грузовых
            if use_max_moment {
                result.inertia_trans_x = result.max_inertia_trans_x;
            } else {
                let volume_max = cache.disp(3).1; // максимальный объем
                if volume >= volume_max * 0.98 {
                    result.inertia_trans_x = 0.;
                }
            }
        } else {
            // Для грузовых цистерн (перевозимых полезный груз, "CompartmentPurpose"="cargo_tank)
            // считаем при крене 5 градусов
            let heel = 5.0 * heel.signum();
            let CompartmentCacheResult {
                inertia_trans_x, ..
            } = self
                .get(heel, trim, volume, epsilon)
                .map_err(|err| error.pass(err))?;
            result.inertia_trans_x = inertia_trans_x;
        }
        //    println!("compartment_cashe {} get_for_dso ok: heel:{heel} volume:{volume} result.volume:{} y:{}", self.dbg, result.volume, result.volume_center.y());
        return Ok(result);
    }    
    /// Получение значения из кэша для заданных условий для расчета равновесного положения
    /// https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter01_initialStability/chapter01_initialStability.md\
    pub fn get(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
    ) -> Result<CompartmentCacheResult, Error> {
        let error = Error::new(self.dbg(), "get");
     /*   println!(
            "{} get start, heel:{heel} trim:{trim} volume:{volume}",
            self.dbg
        );*/
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let coeff = self.coeff.as_ref().ok_or(error.pass("no coeff"))?;
        let (level, result) = get_volume(
            &self.dbg,
            cache,
            &[heel, trim],
            volume / coeff,
            3,
            epsilon,
        ).map_err(|err| error.pass(err))?;
        Ok(CompartmentCacheResult {
            heel,
            trim,
            level,
            volume: result[0] * coeff,
            volume_center: Position::new(result[1], result[2], result[3]),
            inertia_trans_x: result[4] * coeff,
            inertia_long_y: result[5] * coeff,
            max_inertia_trans_x: result[6] * coeff,
            abs_moment: result[7] * coeff,
        })
    }
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
impl LocalCache for HoldCompartmentCache {
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











use crate::{
    algorithm::entities::{
        Bounds, MultipleSingle,
        cache::Cache,
        model_cached::{DisplacementShape, read, save},
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::thread_pool::ThreadPool;
use std::{
    path::PathBuf,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};
///
/// Cache for composite compartment from hold parts
pub struct HoldCompartmentCache {
    dbg: Dbg,
    volume_max: f64,
    caches: OnceLock<Vec<(f64, Option<Cache<f64>>)>>,
    thread_pool: Arc<ThreadPool>,
    exit: Arc<AtomicBool>,
}
//
//
impl HoldCompartmentCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,

        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("HoldCompartmentCache"));
        let cache_path = cache_dir.join(format!("{}", bounds.len_qnt()));
        Self {
            volume_max,
            level_step,
            bounds,
            caches: OnceLock::new(),
            cache_path,
            dbg,
            thread_pool,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Return volume in bounds
    /// cause panic if caches not initialized
    pub fn get(&self, volume: f64, trim: f64, epsilon: f64) -> Result<Vec<f64>, Error> {
        //    println!("jfhufjd {} {volume} {trim} {epsilon}", &self.dbg);
        let error = Error::new(&self.dbg, "get");
        let caches = self.caches.get().ok_or(error.pass("no caches"))?;
        let mut volume_vec = self.get_max_volume().map_err(|err| error.pass(err))?;
        let volume_brutto: f64 = volume_vec.iter().sum();
        let coeff = if volume_brutto > 0. {
            self.volume_max / volume_brutto
        } else {
            1.
        };
        volume_vec.mul_single(coeff);
        if volume >= self.volume_max {
            return Ok(volume_vec);
        }
        let volume = volume / coeff;
        let mut draugth = 0.;
        let mut delta_draugth = 5.;
        let mut last_delta: Option<f64> = Some(-1.);
        let mut values: Vec<f64>;
        let trim_sin = trim.to_radians().sin();
        for _i in 0..100 {
            values = caches
                .iter()
                .map(|(center_x, cache)| match cache {
                    Some(cache) => cache.get(&[(draugth + center_x * trim_sin)])[0],
                    None => 0.,
                })
                .collect();
            let values_sum = values.iter().sum::<f64>();
            let delta = values_sum - volume;
            //    println!("jydfhsh {} {_i} {delta} {values_sum} {volume}", &self.dbg);
            if delta.abs() <= epsilon {
                //           println!("jydfhsh get ok {} {_i} {values_sum} {volume} {epsilon}", &self.dbg);
                values.mul_single(coeff);
                return Ok(values);
            }
            if let Some(last_delta) = last_delta {
                if last_delta.signum() != delta.signum() {
                    delta_draugth = -delta_draugth / 3.;
                }
            }
            draugth += delta_draugth;
            last_delta = Some(delta);
        }
        Err(error.err("no result!"))
    }
    /// Return max volume in bounds
    /// cause panic if caches not initialized
    pub fn get_max_volume(&self) -> Result<Vec<f64>, Error> {
        self.get_max(1)
    }
    /// Return max value in bounds
    /// cause panic if caches not initialized
    fn get_max(&self, index: usize) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "get_max");
        let caches = self.caches.get().ok_or(error.pass("no caches"))?;
        let result = caches
            .iter()
            .map(|(_, cache)| match cache {
                Some(cache) => cache.disp(index).1,
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
        self.clear_exit();
        let errors = self.calculate();
        if errors.is_empty() {
            return Ok(());
        }
        let full_error = errors
            .into_iter()
            .fold("".to_owned(), |acc, err| acc + ", " + &err.to_string());
        Err(Error::new(
            self.dbg.clone(),
            format!("rebuild: {full_error}"),
        ))
    }
    /// инициализация кэшей заранее посчитанными данными
    pub fn init(&self) -> Result<(), Error> {
        let error = Error::new(self.dbg.clone(), "init");
        let mut caches = Vec::new();
        for (i, b) in self.bounds.iter().enumerate() {
            let center_x = b.center().unwrap_or(0.);
            caches.push(
                if let Ok(vals) = read(&self.dbg, &self.cache_path.clone().join(format!("{i}"))) {
                    let cache = Cache::new(&self.dbg);
                    cache
                        .init(vals)
                        .map_err(|err| error.pass_with("cache.init error", err))?;
                    (center_x, Some(cache))
                } else {
                    (center_x, None)
                },
            );
        }
        self.process(caches)
            .map_err(|err| error.pass_with("process_center", err))
    }
    //
    fn calculate(&mut self) -> Vec<Error> {
        let error = Error::new(&self.dbg, "calculate");
        let (data, mut errors) = super::build_cache::BuildHoldCompartmentCache::new(
            &self.dbg,
            self.shape.clone(),
            self.level_step,
            self.bounds.clone(),
            Arc::clone(&self.thread_pool),
            self.exit.clone(),
        )
        .build();
        let mut caches = Vec::new();
        for (i, (dx, v)) in data.into_iter().enumerate() {
            if self.exit.load(Ordering::Relaxed) {
                errors.push(error.err("exit"));
                return errors;
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
                                errors.push(error);
                                return errors;
                            }
                        }
                    }
                    Err(err) => {
                        let error = error.pass_with("cache.init()", err);
                        log::error!("{}", error);
                        errors.push(error);
                        return errors;
                    }
                }
                Some(cache)
            } else {
                None
            };
            caches.push((dx, cache));
        }
        if let Err(err) = self.process(caches) {
            let error = error.pass_with("process_center", err);
            log::error!("{}", error);
            errors.push(error);
        }
        errors
    }
    //
    fn process(&self, caches: Vec<(f64, Option<Cache<f64>>)>) -> Result<(), Error> {
        let error = Error::new(self.dbg.clone(), "process_center");
        let center_x: Vec<_> = caches
            .iter()
            .filter(|(_, v)| v.is_some())
            .map(|(x, _)| *x)
            .collect();
        let qnt = center_x.len();
        let center_x = center_x.iter().sum::<f64>() / qnt as f64;
        self.caches
            .set(
                caches
                    .into_iter()
                    .map(|(x, cache)| (x - center_x, cache))
                    .collect(),
            )
            .map_err(|_| error.err("caches.set"))?;
        //println!("dfsfgukfk {} {center_x} ", &self.dbg);
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
