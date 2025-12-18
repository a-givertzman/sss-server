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

  /*  
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
    /*    println!(
            "{} get start, heel:{heel} trim:{trim} volume:{volume}",
            self.dbg
        );*/
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let coeff = self.coeff.as_ref().ok_or(error.pass("no coeff"))?;
        let volume_ = volume / coeff;
        let (level_min, level_max) = cache.disp(2);
        let (volume_min, volume_max) = cache.disp(3);
        let mut level = volume_min;
        let result = if volume_ <= volume_min {
            // если объем полный или нулевой сразу берем значение
            level = level_min;
            cache.values_disp(&[Some(heel), Some(trim), Some(level_min)]).first().ok_or(error.err(format!("no result! heel:{heel} trim:{trim} level_max:{level_max} trg_volume:{volume_} coeff:{coeff}")))?.to_vec()
        } else if volume_ >= volume_max {
            // если объем полный или нулевой сразу берем значение
            level = level_max;
            cache.values_disp(&[Some(heel), Some(trim), Some(level_max)]).first().ok_or(error.err(format!("no result! heel:{heel} trim:{trim} level_max:{level_max} trg_volume:{volume_} coeff:{coeff}")))?.to_vec()
        } else {
            // ищем значение постепенно приближая объем перебирая уровни заполнения
            let mut current_level = level_max / 2.;
            let mut step = level_max / 4.;
            let mut last_delta_signum = 1.;
            let mut result = Vec::new();
            'volume_loop: for i in 0..=50 {
                let query = [&heel, &trim, &current_level];
                //    println!("compartment_cashe {} get heel:{heel} level:{level}", self.dbg);
                result = cache.get(&query);
                assert!(result.len() >= 6);
                let delta = volume_
                    - result
                        .first()
                        .ok_or(error.pass("no result from cache.get(&query)"))?;
                if last_delta_signum != delta.signum() {
                    step = step * 0.3;
                    last_delta_signum = delta.signum();
                }
                let next_level = current_level + step * delta.signum();
                current_level = next_level.min(level_max).max(level_min);
                //       println!("compartment_cashe {} get i:{i} heel:{heel} level:{level} trg_volume:{volume_} res_volume:{} coeff:{coeff} volume_:{volume_} delta:{delta} y:{}", self.dbg, result[0], result[2]);
                if delta.abs() <= epsilon || i >= 50 || current_level == next_level {
                    //                   println!("compartment_cashe {} result get i:{i} heel:{heel} trg_volume:{volume_} res_volume:{} coeff:{coeff} volume_:{volume_} delta:{delta} y:{}", self.dbg, result[0], result[2]);
                    level = current_level;
                    break 'volume_loop;
                }
            }
            result
        };
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
*/
 /*      /// Получение значения из кэша для заданных условий для расчета равновесного положения
    /// https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter01_initialStability/chapter01_initialStability.md\
    pub fn get(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
    ) -> Result<CompartmentCacheResult, Error> {
        let error = Error::new(self.dbg(), "get");
        println!(
            "{} get start, heel:{heel} trim:{trim} volume:{volume}",
            self.dbg
        );
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let coeff = self.coeff.as_ref().ok_or(error.pass("no coeff"))?;
        let volume_ = volume / coeff;
        let (level_min, level_max) = cache.disp(2);
        let (volume_min, volume_max) = cache.disp(3);
        if volume_ <= volume_min || volume_ >= volume_max {
            let result = cache.values_disp(&[Some(heel), Some(trim), Some(level_max)]).first().ok_or(error.err(format!("no result! heel:{heel} trim:{trim} level_max:{level_max} trg_volume:{volume_} coeff:{coeff}")))?.to_vec();
            return Ok(CompartmentCacheResult {
                heel,
                trim,
                level: level_max,
                volume: result[0] * coeff,
                volume_center: Position::new(result[1], result[2], result[3]),
                inertia_trans_x: result[4] * coeff,
                inertia_long_y: result[5] * coeff,
                max_inertia_trans_x: result[6] * coeff,
                abs_moment: result[7] * coeff,
            });
        }
        let mut level = level_max / 2.;
        let mut step = level_max / 4.;
        let mut last_delta_signum = 1.;
        for i in 0..=50 {
            let query = [&heel, &trim, &level];
            //    println!("compartment_cashe {} get heel:{heel} level:{level}", self.dbg);
            let result = cache.get(&query);
            assert!(result.len() >= 6);
            let delta = volume_
                - result
                    .first()
                    .ok_or(error.pass("no result from cache.get(&query)"))?;
            if last_delta_signum != delta.signum() {
                step = step * 0.3;
                last_delta_signum = delta.signum();
            }
            let next_level = level + step * delta.signum();
            level = next_level.min(level_max).max(level_min);
            //       println!("compartment_cashe {} get i:{i} heel:{heel} level:{level} trg_volume:{volume_} res_volume:{} coeff:{coeff} volume_:{volume_} delta:{delta} y:{}", self.dbg, result[0], result[2]);
            if delta.abs() <= epsilon || i >= 50 || level == next_level {
                //                   println!("compartment_cashe {} result get i:{i} heel:{heel} trg_volume:{volume_} res_volume:{} coeff:{coeff} volume_:{volume_} delta:{delta} y:{}", self.dbg, result[0], result[2]);
                return Ok(CompartmentCacheResult {
                    heel,
                    trim,
                    level,
                    volume: result[0] * coeff,
                    volume_center: Position::new(result[1], result[2], result[3]),
                    inertia_trans_x: result[4] * coeff,
                    inertia_long_y: result[5] * coeff,
                    max_inertia_trans_x: result[6] * coeff,
                    abs_moment: result[7] * coeff,
                });
            }
        }
        Err(error.pass(format!("no result for epsilon:{epsilon}")))
    }*/
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

