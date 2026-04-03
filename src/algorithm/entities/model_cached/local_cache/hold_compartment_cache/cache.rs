use crate::{
    algorithm::entities::{
        Moment, model_cached::{CompartmentCache, CompartmentCacheResult, local_cache::LocalCache},
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use std::sync::atomic::AtomicBool;

///
/// Pre-calculated cache for floating position algorithm.
pub struct HoldCompartmentCache {
    dbg: Dbg,
    /// Минимальный объем отсека из БД (Нетто)
    volume_min: f64,
    /// Максимальный объем отсека из БД (Нетто)
    volume_max: f64,
    /// Минимальный уровень отсека
    level_min: f64,
    /// Максимальный уровень отсека
    level_max: f64,
    /// Список частей отсека
    compartments: Vec<Arc<RwLock<CompartmentCache>>>,
    exit: Arc<AtomicBool>,
}
//
impl HoldCompartmentCache {
    ///
    /// Creates a new instance.
    /// * cache_dir - folder contains all cache files
    /// * volume_max - полный объем из бд
    pub fn new(
        parent: &Dbg,
        code: &String,
        compartments: Vec<Arc<RwLock<CompartmentCache>>>,
    ) -> Result<Self, Error> {
        let dbg = Dbg::new(parent, format!("HoldCompartmentCache_{code}"));
        let error = Error::new(&dbg, "new");
        let level_index = 2;
        let volume_index = 3;
        let (level_min, level_max) = {
            let (_level_min, mut level_max) = (f64::MAX, f64::MIN);
            for compartment in compartments.iter() {
                let compartment = compartment.read();
                let mut current_level_max = compartment.level_max();
                if current_level_max.is_none() {
                    let cache = compartment
                        .cache()
                        .ok_or(error.err("no cache"))?;
                    current_level_max = Some(cache.disp(level_index).1);
                }
                level_max = level_max.max(current_level_max.unwrap());
            }
            (0., level_max)
        };
        let (volume_min, volume_max) = {
            let (volume_min, mut volume_max) = (0., 0.);
            for compartment in compartments.iter() {
                let compartment = compartment.read();
                let mut current_volume_max = compartment.volume_max();
                if current_volume_max.is_none() {
                    let cache = compartment
                        .cache()
                        .ok_or(error.err("no cache"))?;
                    current_volume_max = Some(cache.disp(volume_index).1);
                }
                volume_max += current_volume_max.unwrap();
            }
            (volume_min, volume_max)
        };
//        println!("flkjsdfiklsjfl {dbg} level_min:{level_min}, level_max:{level_max} volume_min:{volume_min} volume_max:{volume_max}");
        Ok(Self {
            dbg,
            volume_min,
            volume_max,
            level_min,
            level_max,
            compartments,
            exit: Arc::new(AtomicBool::new(false)),
        })
    }
    /// Получение значения из кэша для заданных условий для расчета равновесного положения
    /// https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter01_initialStability/chapter01_initialStability.md\
    pub fn get_level(
        &self,
        heel: f64,
        trim: f64,
        volume: f64,
        epsilon: f64,
    ) -> Result<CompartmentCacheResult, Error> {
        let error = Error::new(&self.dbg, "get");
    /*    println!(
            "{} get start, heel:{heel} trim:{trim} volume:{volume}",
            self.dbg
        );*/
        let calc_res = |level: f64| -> Result<CompartmentCacheResult, Error> {
            let mut result = Vec::new();
            for compartment in self.compartments.iter() {
                let current_result = compartment
                    .read()
                    .get_volume(heel, trim, level)
                    .map_err(|err| error.pass(err))?;
                result.push(current_result);
            }
            let (
                volume,
                volume_moment,
                inertia_trans_x,
                inertia_long_y,
                max_inertia_trans_x,
                abs_moment,
            ) = result.iter().fold(
                (0., Moment::zero(), 0., 0., 0., 0.),
                |(
                    acc_volume,
                    acc_volume_moment,
                    acc_inertia_trans_x,
                    acc_inertia_long_y,
                    acc_max_inertia_trans_x,
                    acc_abs_moment,
                ),
                 row| {
                    (
                        acc_volume + row.volume,
                        acc_volume_moment + Moment::from_pos(row.volume_center, row.volume),
                        acc_inertia_trans_x + row.inertia_trans_x,
                        acc_inertia_long_y + row.inertia_long_y,
                        acc_max_inertia_trans_x + row.max_inertia_trans_x,
                        acc_abs_moment + row.abs_moment,
                    )
                },
            );
            Ok(CompartmentCacheResult {
                heel,
                trim,
                level,
                volume,
                volume_center: volume_moment.to_pos(volume),
                inertia_trans_x,
                inertia_long_y,
                max_inertia_trans_x,
                abs_moment,
            })
        };
        if volume <= self.volume_min {
            // целевое значение на нижней границе диапазона, сразу берем значение
            let result = calc_res(self.level_min)?;
            Ok(result)
        } else if volume >= self.volume_max {
            // целевое значение на верхней границе диапазона, сразу берем значение
            let result = calc_res(self.level_max)?;
            Ok(result)
        } else {
            // ищем значение постепенно приближая объем перебирая уровни заполнения
            let mut level = self.level_max / 2.;
            let mut step = self.level_max / 4.;
            let mut last_delta_signum = 1.;
            for i in 0..=50 {
                let result = calc_res(level)?;
                let delta = volume - result.volume;
                if last_delta_signum != delta.signum() {
                    step *= 0.3;
                    last_delta_signum = delta.signum();
                }
                let next_level = (level + step * delta.signum())
                    .min(self.level_max)
                    .max(self.level_min);
        /*        println!(
                    "local_cashe {} get_volume i:{i} heel:{heel} trim:{trim} level:{level} res_volume:{} trg_volume:{volume}",
                    self.dbg, result.volume
                );*/
                if delta.abs() <= epsilon || i >= 50 || level == next_level {
                    return Ok(result);
                }
                level = next_level.min(self.level_max).max(self.level_min);
            }
            Err(error.err("no result!"))
            //      println!("local_cashe {} get_volume result {:?} level:{level} trg_volume:{volume} res:{:?} ", parent, &query, &result);
        }
    }
}
