use crate::{
    algorithm::entities::{
        AddVec, model_cached::CompartmentBoundCache,
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
        compartments: Vec<Arc<RwLock<CompartmentBoundCache>>>,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("HoldCompartmentBoundCache_{code}"));
        let volume_max = compartments
            .iter()
            .map(|v| v.read())
            .flat_map(|v| v.get_max_volume())
            .map(|v| v.into_iter().sum::<f64>())
            .sum();
        Self {
            dbg,
            volume_max,
            compartments,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Return volume in bounds
    /// cause panic if caches not initialized
    pub fn get(&self, volume: f64, trim: f64, epsilon: f64) -> Result<Vec<f64>, Error> {
        //    println!("jfhufjd {} {volume} {trim} {epsilon}", &self.dbg);
        let error = Error::new(&self.dbg, "get");
        let compartments = &self.compartments;
        if volume >= self.volume_max {
            let mut volume_vec = compartments
                .iter()
                .map(|v| v.read())
                .flat_map(|v| v.get_max_volume())
                .fold(Vec::new(), |mut acc, v| {
                    acc.push(v);
                    acc
                });
            let mut acc = volume_vec.pop().ok_or(error.err("empty volumes"))?;
            for v in volume_vec.into_iter() {
                acc.add_vec(&v).map_err(|err| error.pass(err))?;
            }
            return Ok(acc);
        }
        let mut draugth = 0.;
        let mut delta_draugth = 5.;
        let mut last_delta: Option<f64> = Some(-1.);
        let mut values: Vec<f64>;
        for _i in 0..100 {
            let mut volume_vec = compartments.iter().map(|v| v.read())
                .flat_map(|v| v.get_from_level(draugth, trim))
                .fold(Vec::new(), |mut acc, v| {
                    acc.push(v);
                    acc
                });
            values = volume_vec.pop().ok_or(error.err("empty volumes"))?;
            for v in volume_vec.into_iter() {
                values.add_vec(&v).map_err(|err| error.pass(err))?;
            }
            let values_sum = values.iter().sum::<f64>();
            let delta = values_sum - volume;
            //    println!("jydfhsh {} {_i} {delta} {values_sum} {volume}", &self.dbg);
            if delta.abs() <= epsilon {
                //           println!("jydfhsh get ok {} {_i} {values_sum} {volume} {epsilon}", &self.dbg);
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
}
