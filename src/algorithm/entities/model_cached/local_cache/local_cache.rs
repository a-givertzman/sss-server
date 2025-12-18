use crate::algorithm::entities::{cache::Cache, model_cached::read};
use sal_core::{dbg::Dbg, error::Error};
use std::path::PathBuf;

///
/// A common trait for caches, which work with file systems.
/*
pub trait LocalCache {
    // ///
    // /// Builds and stores the cache dataset.
    // ///
    // /// This method spawns a worker thread internally and returns its handler.
    // /// Setting `exit` to _true_ at the caller side stops the worker.
    // fn calculate(
    //     &self,
    //     exit: Arc<AtomicBool>,
    // ) -> Vec<Error>;
    ///
    /// Returns approximated values based on given set.
    fn get(&self, approx_vals: &[Option<f64>]) -> Result<Vec<f64>, Error>;
    ///
    /// Rebuilds a cache
    /// - takes new model
    /// - do calculations
    /// - stores calculated table
    /// - loads recalculated table
    fn rebuild(&mut self) -> Result<(), Error>;
    ///
    /// Sends exit signal to hawy calculations
    fn exit(&self);
}
    */

//
//
pub(crate) trait LocalCache {
    fn dbg(&self) -> &Dbg;

    fn cache_path(&self) -> PathBuf;

    fn cache(&self) -> Option<&Cache<f64>>;

    fn set_cache(&mut self, cache: Cache<f64>);
    ///
    /// Sends exit signal to hawy calculations
    fn exit(&self);
    ///
    /// Remove exit signal
    fn clear_exit(&self);
    ///
    /// Builds and stores the cache dataset.
    ///
    /// This method spawns a worker thread internally and returns its handler.
    fn calculate(&mut self) -> Vec<Error>;
    ///
    /// Returns approximated values based on given set.
    // TODO получениеf
    fn get(&self, approx_vals: &[f64]) -> Result<Vec<f64>, Error> {
        let error = Error::new(self.dbg(), "get");
        Ok(self
            .cache()
            .as_ref()
            .ok_or(error.pass("no cache"))?
            .get(approx_vals))
    }
    /// Rebuilds a cache
    /// - takes new model
    /// - do calculations
    /// - stores calculated table
    /// - loads recalculated table
    fn rebuild(&mut self) -> Result<(), Error> {
        self.clear_exit();
        let errors = self.calculate();
        if !errors.is_empty() {
            return Err(Error::new(self.dbg(), "rebuild").pass_with(
                "calculate",
                errors
                    .iter()
                    .fold(String::new(), |acc, err| acc + &format!(" error: {err}")),
            ));
        }
        Ok(())
    }
    /// инициализация кэша заранее посчитанными данными
    fn init(&mut self) -> Result<(), Error> {
        let error = Error::new(self.dbg(), "init");
        let vals = read(self.dbg(), &self.cache_path())
            .map_err(|err| error.pass_with(format!("read cache data error"), err))?;
        let cache = Cache::new(self.dbg());
        cache
            .init(vals)
            .map_err(|err| error.pass_with("cache.init error", err))?;
        self.set_cache(cache);
        Ok(())
    }
}

/// Получение значения из кэша для заданных условий и объема.
/// На входе параметры без уровня, уровень подбирается к объему.
/// Уровень должен быть последним ключем и не входить в query.
pub fn get_volume(
    parent: &Dbg,
    cache: &Cache<f64>,
    query: &[f64],
    volume: f64,
    volume_index: usize,
    epsilon: f64,
) -> Result<(f64, Vec<f64>), Error> {
    let error = Error::new(parent, "get");
  /*  println!(
        "{} get_volume start, query:{:?} volume:{volume} target_index:{volume_index}",
        parent, query,
    );*/
    let (level_min, level_max) = cache.disp(query.len());
    let (volume_min, volume_max) = cache.disp(volume_index);
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
            //    println!("compartment_cashe {} get heel:{heel} level:{level}", self.dbg);
            result = cache.get(&query);
            assert!(result.len() >= volume_index);
            let delta = volume - result[volume_index - query.len()];
            if last_delta_signum != delta.signum() {
                step = step * 0.3;
                last_delta_signum = delta.signum();
            }
            let next_level = (level + step * delta.signum()).min(level_max).max(level_min);
       //     println!("local_cashe {} get_volume i:{i} heel:{} trim:{} level:{level} res_volume:{} trg_volume:{volume}", parent, query[0], query[1], result[0]);
            if delta.abs() <= epsilon || i >= 50 || level == next_level {          
                break 'volume_loop;
            }
            level = next_level.min(level_max).max(level_min);
        }
    //    println!("local_cashe {} get_volume result {:?} level:{level} trg_volume:{volume} res:{:?} ", parent, &query, &result);
        (level, result)
    };
    Ok((level, result))
}
