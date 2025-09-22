use crate::{
    algorithm::entities::{cache::Cache, model_cached::read},
    kernel::types::{Arc, RwLock},
};
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
        match self.calculate().first() {
            Some(err) => Err(Error::new(self.dbg(), "rebuild").pass(err.to_owned())),
            None => Ok(()),
        }
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
