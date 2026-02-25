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
    // TODO получение
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
pub fn get_from_volume(
    parent: &Dbg,
    cache: &Cache<f64>,
    query: &[f64],
    volume: f64,
    level_max: Option<f64>,
    volume_max: Option<f64>,
    volume_index: usize,
    epsilon: f64,
) -> Result<(f64, Vec<f64>), Error> {
    let error = Error::new(parent, "get_from_volume");
    /*  println!(
        "{} get_from_volume start, query:{:?} volume:{volume} target_index:{volume_index}",
        parent, query,
    );*/
    let volume_max = volume_max
        .or_else(|| Some(cache.disp(volume_index).1))
        .ok_or(error.err("no volume_max"))?;
    let (mut level, mut result) = if volume <= 0. {
        // целевое значение на нижней границе диапазона, сразу берем значение
        let (level_min, _) = cache.disp(query.len());
        let mut query: Vec<_> = query.iter().map(|&v| Some(v)).collect();
        query.push(Some(level_min));
        let result = cache
            .values_disp(&query)
            .first()
            .ok_or(error.err(format!("no result!")))?
            .to_vec();
        (0., result)
    } else if volume >= volume_max {
        // целевое значение на верхней границе диапазона, сразу берем значение
        let (_, level_max) = cache.disp(query.len());
        let mut query: Vec<_> = query.iter().map(|&v| Some(v)).collect();
        query.push(Some(level_max));
        let result = cache
            .values_disp(&query)
            .first()
            .ok_or(error.err(format!("no result!")))?
            .to_vec();
        (level_max, result)
    } else {
        // ищем значение постепенно приближая объем перебирая уровни заполнения
        let (level_min, level_max) = cache.disp(query.len());
        let mut level = level_max / 2.;
        let mut step = level_max / 5.;
        let mut last_delta_signum = 1.;
        let mut result = Vec::new();
        'volume_loop: for i in 0..=50 {
            let mut query: Vec<_> = query.to_vec();
            query.push(level);
            result = cache.get(&query);
            assert!(result.len() > volume_index);
            let delta = volume - result[volume_index - query.len()];
            if last_delta_signum != delta.signum() {
                step = step * 0.3;
                last_delta_signum = delta.signum();
            }
            let next_level = (level + step * delta.signum())
                .min(level_max)
                .max(level_min);
            //          println!("local_cashe {} get_from_volume i:{i} heel:{} trim:{} level:{level} res_volume:{} trg_volume:{volume}, index:{}", parent, query[0], query[1], result[0], volume_index - query.len());
            if delta.abs() <= epsilon || i >= 50 || level == next_level {
                break 'volume_loop;
            }
            level = next_level;
        }
        //      println!("local_cashe {} get_from_volume result {:?} level:{level} trg_volume:{volume} res:{:?} ", parent, &query, &result);
        (level, result)
    };
    result[0] = volume;
    if let Some(level_max) = level_max {
        level = level.max(0.).min(level_max);
    }
    Ok((level.max(0.), result))
}

/// Получение значения из кэша для заданных условий и объема.
/// На входе параметры без уровня, уровень подбирается к объему.
/// Уровень должен быть последним ключем и не входить в query.
pub fn get_from_level(
    parent: &Dbg,
    cache: &Cache<f64>,
    query: &[f64],
    level: f64,
    volume_max: Option<f64>,
    volume_index: usize,
) -> Result<Vec<f64>, Error> {
    let error = Error::new(parent, "get_from_level");
    /*  println!(
        "{} get_from_level start, query:{:?} volume:{volume} target_index:{volume_index}",
        parent, query,
    );*/
    let volume_max = volume_max
        .or_else(|| Some(cache.disp(volume_index).1))
        .ok_or(error.err("no volume_max"))?;
    let (level_min, level_max) = cache.disp(query.len());
    //   dbg!(level_min, level_max, volume_min, volume_max);
    let mut result = if level <= level_min {
        // целевое значение на нижней границе диапазона, сразу берем значение
        let mut query: Vec<_> = query.iter().map(|&v| Some(v)).collect();
        query.push(Some(level_min));
        cache
            .values_disp(&query)
            .first()
            .ok_or(error.err(format!("no result!")))?
            .to_vec()
    } else if level >= level_max {
        // целевое значение на верхней границе диапазона, сразу берем значение
        let mut query: Vec<_> = query.iter().map(|&v| Some(v)).collect();
        query.push(Some(level_max));
        cache
            .values_disp(&query)
            .first()
            .ok_or(error.err(format!("no result!")))?
            .to_vec()
    } else {
        let mut query: Vec<_> = query.to_vec();
        query.push(level);
        cache.get(&query)
    };
    // при крене/дифференте объем и уровень могут быть отрицательными для заданного уровня
    // но для расчета это не имеет смысла, обнуляем в таком случае
    result[volume_index] = result[volume_index].max(0.).min(volume_max);
    Ok(result)
}
