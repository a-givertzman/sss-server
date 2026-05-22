use crate::algorithm::entities::{Cache, model_cached::read};
use sal_3dlib_core::math::Bounds;
use sal_core::{dbg::Dbg, error::Error};
use std::{path::PathBuf, sync::OnceLock};
///
/// Pre-calculated cache for bounds
pub struct DisplacementBoundCache {
    dbg: Dbg,
    cache_path: PathBuf,
    center_x: f64,
    bounds: Bounds,
    ///
    /// Cache read from `self.file_path`.
    caches: OnceLock<Vec<(f64, Option<Cache<f64>>)>>,
}
//
//
impl DisplacementBoundCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        cache_dir: PathBuf,
        center_x: f64,
        bounds: Bounds,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("DisplacementBoundCache_{:.3}", center_x));
        let cache_path = cache_dir.join(format!("{}", bounds.len_qnt()));
        Self {
            center_x,
            bounds,
            caches: OnceLock::new(),
            cache_path,
            dbg,
        }
    }
    /// Return volume in bounds
    /// cause panic if caches not initialized
    pub fn get(&self, trim: f64, draught_mid: f64) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "get");
        let caches = self.caches.get().ok_or(error.pass("no caches"))?;
        //    let delta_draught = trim.to_radians().sin()*self.length_lbp;
        let result = caches
            .iter()
            .map(|(center_x, cache)| match cache {
                Some(cache) => {
                    //            let draught = draught_mid + delta_draught * (dx - self.length_lbp / 2. + self.center_x);
                    let draught = draught_mid + center_x * trim.to_radians().sin();
                    //       dbg!(draught_mid, dx, draught);
                    cache.get(&[draught])[0]
                }
                None => 0.,
            })
            .collect();
        Ok(result)
    }
    /// Return max volume in bounds
    /// cause panic if caches not initialized
    pub fn get_max(&self) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "get_max");
        let caches = self.caches.get().ok_or(error.pass("no caches"))?;
        let result = caches
            .iter()
            .map(|(_, cache)| match cache {
                Some(cache) => cache.disp(1).1,
                None => 0.,
            })
            .collect();
        Ok(result)
    }
    /// инициализация кэшей заранее посчитанными данными
    pub fn init(&self) -> Result<(), Error> {
        let error = Error::new(self.dbg.clone(), "init");
        let mut caches = Vec::new();
        for (i, bound) in self.bounds.iter().enumerate() {
            let cache =
                if let Ok(vals) = read(&self.dbg, &self.cache_path.clone().join(format!("{i}"))) {
                    let cache = Cache::new(&self.dbg);
                    cache
                        .init(vals)
                        .map_err(|err| error.pass_with("cache.init error", err))?;
                    Some(cache)
                } else {
                    None
                };
            let center = bound.center().ok_or(error.err("bound.center()"))? - self.center_x;
            caches.push((center, cache));
        }
        self.caches
            .set(caches)
            .map_err(|_| error.err("caches.set"))?;
        Ok(())
    }
}
