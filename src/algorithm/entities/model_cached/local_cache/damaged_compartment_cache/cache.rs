use crate::{
    algorithm::entities::{
        Position,
        Cache,
        model_cached::{get_from_level, local_cache::LocalCache},
    },
};
use sal_core::{dbg::Dbg, error::Error};
use std::{
    path::{Path, PathBuf},
};
///
/// Pre-calculated cache for floating position algorithm.
pub struct DamagedCompartmentCache {
    dbg: Dbg,
    cache_path: PathBuf,
    draught_min: f64,
    draught_max: f64,
    ///
    /// Cache read from `self.file_path`.
    cache: Option<Cache<f64>>,
}
//
//
impl DamagedCompartmentCache {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        parent: &Dbg,
        cache_dir: impl AsRef<Path>,
        compartment_id: String,
        draught_min: f64,
        draught_max: f64,
    ) -> Self {
        let dbg = Dbg::new(parent, format!("DamagedCompartmentCache_{compartment_id}"));
        Self {
            draught_min,
            draught_max,
            cache: None,
            cache_path: cache_dir.as_ref().join(compartment_id),
            dbg,
        }
    }
    /// Return (volume, center of volume)
    pub fn get(&self, heel: f64, trim: f64, draught: f64) -> Result<(f64, Position), Error> {
        let error = Error::new(self.dbg(), "get");
        let cache = self.cache.as_ref().ok_or(error.pass("no cache"))?;
        let query = [heel, trim];
        let result = get_from_level(&self.dbg, cache, &query, draught, None, 3)
            .map_err(|err| error.pass_with("get_from_level", err))?;
        Ok((result[0], Position::new(result[1], result[2], result[3])))
    }
}
//
//
impl LocalCache for DamagedCompartmentCache {
    //
    fn dbg(&self) -> &Dbg {
        &self.dbg
    }
    //
    fn cache_path(&self) -> PathBuf {
        self.cache_path.clone()
    }
    //
    fn cache(&self) -> Option<&Cache<f64>> {
        self.cache.as_ref()
    }

    fn set_cache(&mut self, cache: Cache<f64>) {
        self.cache = Some(cache);
    }
}
