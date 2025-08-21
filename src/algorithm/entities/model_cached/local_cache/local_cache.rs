use std::{path::PathBuf, sync::atomic::{AtomicBool, Ordering}};

use sal_core::{dbg::Dbg, error::Error};

use crate::{algorithm::entities::{cache::Cache, model_cached::read}, kernel::types::{Arc, RwLock}};

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
pub trait LocalCache {
    fn dbg(&self) -> &Dbg;

    fn cache_path(&self) -> &PathBuf;

    fn cache(&self) -> &Arc<RwLock<Option<Cache<f64>>>>;
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
    fn get(&self, approx_vals: &[Option<f64>]) -> Result<Vec<f64>, Error> {
        let error = Error::new(self.dbg(), "get");
        if self.cache().read().is_none() {
            let cache = Cache::new(self.dbg());
            let vals = read(self.dbg(), self.cache_path())
                .map_err(|err| error.pass_with("read cache data error", err))?;
            cache
                .init(vals)
                .map_err(|err| error.pass_with("cache.init error", err))?;
            let _ = self.cache().write().insert(cache);
        }
        Ok(self
            .cache()
            .read()
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
        match self.calculate().first() {
            Some(err) => Err(Error::new(self.dbg(), "rebuild").pass(err.to_owned())),
            None => Ok(()),
        }
    }
}
