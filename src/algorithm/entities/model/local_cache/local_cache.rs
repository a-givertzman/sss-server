use std::sync::{atomic::AtomicBool, Arc};
use sal_core::error::Error;

///
/// A common trait for caches, which work with file systems.
pub trait LocalCache {
    ///
    /// Builds and stores the cache dataset.
    ///
    /// This method spawns a worker thread internally and returns its handler.
    /// Setting `exit` to _true_ at the caller side stops the worker.
    fn calculate(
        &self,
        exit: Arc<AtomicBool>,
    ) -> Vec<Error>;
    ///
    /// Returns approximated values based on given set.
    fn get(&self, approx_vals: &[Option<f64>]) -> Option<Vec<Vec<f64>>>;
    ///
    /// Reloads caches.
    ///
    /// Typicaly, calling of this method should follow a call of [LocalCache::calculate].
    fn reload(&mut self);
}
