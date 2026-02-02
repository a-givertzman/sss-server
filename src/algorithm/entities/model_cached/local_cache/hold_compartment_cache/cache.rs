use crate::{
    algorithm::entities::{
        AddVec, Bounds, Curve, ICurve, Position, cache::Cache, model_cached::{
            CompartmentBoundCache, CompartmentCacheResult, DisplacementShape, get_volume, local_cache::LocalCache, save
        }
    },
    kernel::types::{Arc, RwLock},
};
use sal_core::{dbg::Dbg, error::Error};

///
/// Pre-calculated cache for floating position algorithm.
pub struct HoldCompartmentCache {
    dbg: Dbg,
    /// Максимальный объем отсека из БД (Нетто)
    volume_max: f64,
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
        code: String,
        compartments: Vec<Arc<RwLock<CompartmentCache>>>,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("HoldCompartmentCache_{code}"));
        let volume_max = 
        compartments
            .iter()
            .map(|v| v.read())
            .map(|v| *v.volume_max()).sum();
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
        let error = Error::new(self.dbg(), "get");
     /*   println!(
            "{} get start, heel:{heel} trim:{trim} volume:{volume}",
            self.dbg
        );*/
        let level_index = 2;
        let volume_index = 3;
        let mut query = vec![heel, trim];
        let (level_min, level_max) = 
            self.compartments.iter()
                .fold((f64::MAX, f64::MIN), |(level_min, level_max), c|{ 
                    let (current_level_min, current_level_max) = c.disp(level_index);
                    (level_min.min(current_level_min), level_max.max(current_level_max))
                } );
        let (volume_min, volume_max) = 
            self.compartments.iter()
                .fold((0., 0.), |(volume_min_sum, volume_max_sum), c|{ 
                    let (volume_min, volume_max) = c.disp(volume_index);
                    (volume_min_sum + volume_min, volume_max_sum + volume_max)
                } );
        let (level, result) = if volume <= volume_min {
            // целевое значение на нижней границе диапазона, сразу берем значение
            let mut query: Vec<_> = query.iter().map(|&v| Some(v)).collect();
            query.push(Some(level_min));
            (level_min, cache.values_disp(&query).first().ok_or(error.err(format!("no result!")))?.to_vec())
        } else if volume >= volume_max {
            // целевое значение на верхней границе диапазона, сразу берем значение
            let mut query: Vec<_> = query.iter().map(|&v| Some(v)).collect();
            query.push(Some(level_max));
            (level_max, cache.values_disp(&query).first().ok_or(error.err(format!("no result!")))?.to_vec())
        } else {
            // ищем значение постепенно приближая объем перебирая уровни заполнения
            let mut level = level_max / 2.;
            let mut step = level_max / 4.;
            let mut last_delta_signum = 1.;
            let mut result = Vec::new();
            'volume_loop: for i in 0..=50 {
                let mut query: Vec<_> = query.to_vec();
                query.push(level);
                let result = self.compartments.iter().fold([0.; 8].to_vec(), |result, c| {
                    let current_result = c.read().get(&query);
                    assert!(current_result.len() == result.len());
                    result.add_vec(current_result).unwrap();
                    result
                });
                let delta = volume - result[volume_index - query.len()];
                if last_delta_signum != delta.signum() {
                    step = step * 0.3;
                    last_delta_signum = delta.signum();
                }
                let next_level = (level + step * delta.signum()).min(level_max).max(level_min);
    //          println!("local_cashe {} get_volume i:{i} heel:{} trim:{} level:{level} res_volume:{} trg_volume:{volume}, index:{}", parent, query[0], query[1], result[0], volume_index - query.len());
                if delta.abs() <= epsilon || i >= 50 || level == next_level {          
                    break 'volume_loop;
                }
                level = next_level.min(level_max).max(level_min);
            }
    //      println!("local_cashe {} get_volume result {:?} level:{level} trg_volume:{volume} res:{:?} ", parent, &query, &result);
            (level, result)
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
}
