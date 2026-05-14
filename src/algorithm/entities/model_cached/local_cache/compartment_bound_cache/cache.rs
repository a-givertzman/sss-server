use crate::{
    algorithm::entities::{
        Bounds, MultipleSingle,
        cache::Cache,
        model_cached::read,
    },
};
use sal_core::{dbg::Dbg, error::Error};
use std::{
    path::PathBuf,
    sync::{
        OnceLock,
    },
};
///
/// Pre-calculated cache for bounds
pub struct CompartmentBoundCache {
    dbg: Dbg,
    cache_path: PathBuf,
    volume_max: f64,
    /// [коэффициент проницаемости](https://github.com/a-givertzman/sss/blob/master/design/algorithm-simply/part02_mass/chapter04_volumeNetto.md)
    coeff: Option<f64>,
    bounds: Bounds,
    /// Cache read from `self.file_path`.
    caches: OnceLock<Vec<(f64, Option<Cache<f64>>)>>,
}
//
//
impl CompartmentBoundCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        volume_max: f64,
        cache_dir: PathBuf,
        bounds: Bounds,
    ) -> Self {
        let dbg = Dbg::new(parent, "CompartmentBoundCache".to_string());
        let cache_path = cache_dir.join(format!("{}", bounds.len_qnt()));
        Self {
            volume_max,
            coeff: None,
            bounds,
            caches: OnceLock::new(),
            cache_path,
            dbg,
        }
    }
    /// Return volume in bounds
    /// cause panic if caches not initialized
    pub fn get_from_volume(&self, volume: f64, trim: f64, epsilon: f64) -> Result<Vec<f64>, Error> {
        //    println!("jfhufjd {} {volume} {trim} {epsilon}", &self.dbg);
        let error = Error::new(&self.dbg, "get_from_volume");        
        if volume >= self.volume_max {
            let volume_vec = self.get_max_volume().map_err(|err| error.pass(err))?;    
            return Ok(volume_vec);
        }
        let caches = self.caches.get().ok_or(error.pass("no caches"))?; 
        let coeff = self.coeff.ok_or(error.err("no coeff"))?;       
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
            if let Some(last_delta) = last_delta
                && last_delta.signum() != delta.signum() {
                    delta_draugth = -delta_draugth / 3.;
                }
            draugth += delta_draugth;
            last_delta = Some(delta);
        }
        Err(error.err("no result!"))
    }
    /// Return volume in bounds
    /// cause panic if caches not initialized
    pub fn get_from_level(&self, level: f64, trim: f64) -> Result<Vec<f64>, Error> {
        //    println!("jfhufjd {} {volume} {trim} {epsilon}", &self.dbg);
        let error = Error::new(&self.dbg, "get_from_level");
        let caches = self.caches.get().ok_or(error.pass("no caches"))?;
        let coeff = self.coeff.ok_or(error.err("no coeff"))?;
        let mut values: Vec<f64>;
        let trim_sin = trim.to_radians().sin();
        values = caches
            .iter()
            .map(|(center_x, cache)| match cache {
                Some(cache) => cache.get(&[(level + center_x * trim_sin)])[0],
                None => 0.,
            })
            .collect();
        values.mul_single(coeff);
        Ok(values)
    }
    /// Return max volume in bounds
    /// cause panic if caches not initialized
    pub fn get_max_volume(&self) -> Result<Vec<f64>, Error> {
        let error = Error::new(&self.dbg, "get_max_volume");
        let mut volume_vec = self.get_max(1).map_err(|err| error.pass(err))?;
        let coeff = self.coeff.ok_or(error.err("no coeff"))?;
        volume_vec.mul_single(coeff);     
        Ok(volume_vec)
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
    /// Инициализация кэшей заранее посчитанными данными
    pub fn init(&mut self) -> Result<(), Error> {
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
        let volume_brutto: f64 = caches
            .iter()
            .map(|(_, cache)| match cache {
                Some(cache) => cache.disp(1).1,
                None => 0.,
            })
            .sum();
        let coeff = if volume_brutto > 0. {
            self.volume_max / volume_brutto
        } else {
            1.
        };
        self.coeff = Some(coeff);
        self.process(caches)
            .map_err(|err| error.pass_with("process_center", err))
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
}
