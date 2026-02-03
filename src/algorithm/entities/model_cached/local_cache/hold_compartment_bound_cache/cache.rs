use crate::{
    algorithm::entities::{
        Position,
        model_cached::{CompartmentBoundCache, CompartmentCacheResult, local_cache::LocalCache},
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};
use std::sync::atomic::AtomicBool;

///
/// Pre-calculated cache for floating position algorithm.
pub struct HoldCompartmentBoundCache {
    dbg: Dbg,
    /// Максимальный объем отсека из БД (Нетто)
    volume_max: f64,
    compartments: Vec<Arc<RwLock<CompartmentBoundCache>>>,
    exit: Arc<AtomicBool>,
}
//
impl HoldCompartmentBoundCache {
    ///
    /// Creates a new instance.
    /// * cache_dir - folder contains all cache files
    /// * volume_max - полный объем из бд
    pub fn new(
        parent: &Dbg,
        code: &String,
        compartments: Vec<Arc<RwLock<CompartmentCache>>>,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("HoldCompartmentBoundCache_{code}"));
        let volume_max = compartments
            .iter()
            .map(|v| v.read())
            .map(|v| v.volume_max().unwrap_or(0.))
            .sum();
        Self {
            dbg,
            volume_max,
            compartments,
            exit: Arc::new(AtomicBool::new(false)),
        }
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
        /*   println!(
            "{} get start, heel:{heel} trim:{trim} volume:{volume}",
            self.dbg
        );*/
        let level_index = 2;
        let volume_index = 3;
        let (level_min, level_max) = {
            let (mut level_min, mut level_max) = (f64::MAX, f64::MIN);
            for compartment in self.compartments.iter() {
                let (current_level_min, current_level_max) = compartment
                    .read()
                    .cache()
                    .ok_or(error.err("no cache"))?
                    .disp(level_index);
                level_min = level_min.min(current_level_min);
                level_max = level_max.max(current_level_max);
            }
            (level_min, level_max)
        };
        let (volume_min, volume_max) = {
            let (mut volume_min, mut volume_max) = (f64::MAX, f64::MIN);
            for compartment in self.compartments.iter() {
                let (current_volume_min, current_volume_max) = compartment
                    .read()
                    .cache()
                    .ok_or(error.err("no cache"))?
                    .disp(volume_index);
                volume_min += current_volume_min;
                volume_max += current_volume_max;
            }
            (volume_min, volume_max)
        };
        let calc_res = |level: f64| -> Result<CompartmentCacheResult, Error> {
            let mut result = Vec::new();
            for compartment in self.compartments.iter() {
                let current_result = compartment
                    .read()
                    .get_volume(heel, trim, level)
                    .map_err(|err| error.pass(err))?;
                result.push(current_result);
            }
            let result = result.iter().fold(
                CompartmentCacheResult {
                    heel: heel,
                    trim: trim,
                    level: level,
                    volume: 0.,
                    volume_center: Position::zero(),
                    inertia_trans_x: 0.,
                    inertia_long_y: 0.,
                    max_inertia_trans_x: 0.,
                    abs_moment: 0.,
                },
                |mut acc, row| {
                    acc.volume += row.volume;
                    acc.volume_center += row.volume_center;
                    acc.inertia_trans_x += row.inertia_trans_x;
                    acc.inertia_long_y += row.inertia_long_y;
                    acc.max_inertia_trans_x += row.max_inertia_trans_x;
                    acc.abs_moment += row.abs_moment;
                    acc
                },
            );
            Ok(result)
        };
        if volume <= volume_min {
            // целевое значение на нижней границе диапазона, сразу берем значение
            let result = calc_res(level_min)?;
            Ok(result)
        } else if volume >= volume_max {
            // целевое значение на верхней границе диапазона, сразу берем значение
            let result = calc_res(level_max)?;
            Ok(result)
        } else {
            // ищем значение постепенно приближая объем перебирая уровни заполнения
            let mut level = level_max / 2.;
            let mut step = level_max / 4.;
            let mut last_delta_signum = 1.;
            for i in 0..=50 {
                let result = calc_res(level)?;
                let delta = volume - result.volume;
                if last_delta_signum != delta.signum() {
                    step = step * 0.3;
                    last_delta_signum = delta.signum();
                }
                let next_level = (level + step * delta.signum())
                    .min(level_max)
                    .max(level_min);
                //          println!("local_cashe {} get_volume i:{i} heel:{} trim:{} level:{level} res_volume:{} trg_volume:{volume}, index:{}", parent, query[0], query[1], result[0], volume_index - query.len());
                if delta.abs() <= epsilon || i >= 50 || level == next_level {
                    return Ok(result);
                }
                level = next_level.min(level_max).max(level_min);
            }
            Err(error.err("no result!"))
            //      println!("local_cashe {} get_volume result {:?} level:{level} trg_volume:{volume} res:{:?} ", parent, &query, &result);
        }
    }
}
