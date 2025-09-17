use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler},
};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    algorithm::entities::{Bounds, model_cached::DisplacementShape},
    kernel::types::{Arc, RwLock},
};
///
/// Provides logic to calculate and store cache used by [super::BoundCacheCache].
pub struct BuildBoundCache {
    dbg: Dbg,
    shape: Arc<RwLock<DisplacementShape>>,
    draught_step: f64,
    bounds: Bounds,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl BuildBoundCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        draught_step: f64,
        bounds: Bounds,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
    ) -> Self {
        debug_assert!(draught_step > 0.);
        Self {
            dbg: Dbg::new(parent, "BuildBoundCache"),
            shape: shape.clone(),
            draught_step,
            bounds,
            scheduler,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [BoundCache::calculate].
    ///
    /// results: [[draught, volume]]
    pub fn build(self) -> Result<Vec<(f64, Option<Vec<(f64, f64)>>)>, Error> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let draft_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let errors = Arc::new(Stack::new());
        let shape = self.shape.clone();
        for bound in self.bounds.iter() {
            // _true_ if the caller has requisted to exit.
            // Note that in this case the file may be partially filled.
            if self.exit.load(Ordering::SeqCst) {
                break;
            }
            let draft_results = draft_results.clone();
            let _errors = errors.clone();
            let _error = error.clone();
            let shape = shape.clone();
            let bound = bound.clone();
            let center = match bound.center() {
                Some(center) => center,
                None => {
                    let error = error.err("bound.center()");
                    log::error!("{:?}", &error);
                    errors.push(Err(error));
                    continue;
                }
            };
            let step = self.draught_step;
            let handle = self
                .scheduler
                .spawn(move || {
                    let guard = shape.read();
                    match guard.part(&bound) {
                        Ok(shape) => match shape {
                            Some(shape) => draft_results
                                .push((center, Some(shape.displacement_by_steps(step)))),
                            None => draft_results
                                .push((center, None)),
                        }
                        Err(err) => {
                            let error = _error.pass_with(format!("task center:{center} guard.part"), err.to_string());
                            log::error!("{}", error);
                            _errors.push(Err(error));
                        },
                    }
                    Ok(())
                })
                .map_err(|err| {
                    error.pass_with(format!("spawn task bound:{:?}", bound), err.to_string())
                });
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => {
                    let error = error.pass_with("task handle", err.to_string());
                    log::error!("{}", error);
                    errors.push(Err(error));
                }
            };
        }
        for task in tasks {
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(Err(error));
            }
        }
        while !draft_results.is_empty() {
            if let Some((dx, result)) = draft_results.pop() {
                match result {
                    Some(result) => match result {
                        Ok(result) => results.push((dx, Some(result))),
                        Err(err) => {
                            let error =
                                error.pass_with(format!("result, dx:{dx}"), err.to_string());
                            log::error!("{}", error);
                            errors.push(Err(error));
                        }
                    },
                    None => results.push((dx, None)),
                };
            }
        }
        //   dbg!(&results);
        if let Some(error) = errors.pop() {
            return error.clone();
        }
        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Ok(results)
    }
}
