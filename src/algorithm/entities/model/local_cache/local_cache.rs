use sal_core::error::Error;

///
/// A common trait for caches, which work with file systems.
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
    fn rebuild(&self) -> Result<(), Error>;
    ///
    /// Sends exit signal to hawy calculations
    fn exit(&self);
}
