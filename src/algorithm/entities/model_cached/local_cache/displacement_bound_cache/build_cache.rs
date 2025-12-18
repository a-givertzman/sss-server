use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, ThreadPool},
};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    algorithm::entities::{Bounds, model_cached::DisplacementShape},
    kernel::types::{Arc, RwLock},
};
///
/// Provides logic to calculate and store cache used by [super::DisplacementBoundCache].
pub struct BuildDisplacementBoundCache {
    dbg: Dbg,
    shape: Arc<RwLock<DisplacementShape>>,
    level_step: f64,
    bounds: Bounds,
    thread_pool: Arc<ThreadPool>,
    exit: Arc<AtomicBool>,
}
//
//
impl BuildDisplacementBoundCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        level_step: f64,
        bounds: Bounds,
        thread_pool: Arc<ThreadPool>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        debug_assert!(level_step > 0.);
        Self {
            dbg: Dbg::new(parent, "BuildDisplacementBoundCache"),
            shape: shape.clone(),
            level_step,
            bounds,
            thread_pool,
            exit,
        }
    }
    /// Построение кэшей
    pub fn build(self) -> (Vec<(f64, Option<Vec<(f64, f64)>>)>, Vec<Error>) {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let (results, errors) = self._build();
        let mut vec_results = Vec::new();
        while !results.is_empty() {
            if let Some((dx, result)) = results.pop() {
                match result {
                    Some(result) => match result {
                        Ok(result) => vec_results.push((dx, Some(result))),
                        Err(err) => {
                            let error = error.pass_with(format!("result, dx:{dx}"), err);
                            log::error!("{:?}", &error);
                            errors.push(error);
                        }
                    },
                    None => vec_results.push((dx, None)),
                };
            }
        }
        let mut vec_errors = Vec::new();
        while !errors.is_empty() {
            if let Some(error) = errors.pop() {
                vec_errors.push(error);
            }
        }
        vec_results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        (vec_results, vec_errors)
    }

    pub fn _build(
        self,
    ) -> (
        Arc<Stack<(f64, Option<Result<Vec<(f64, f64)>, Error>>)>>,
        Arc<Stack<Error>>,
    ) {
        log::info!("{}._build | Starting _build", &self.dbg);
        let error = Error::new(&self.dbg, "_build");
        let mut tasks: VecDeque<JoinHandle<_>> = VecDeque::new();
        let results = Arc::new(Stack::new());
        let errors = Arc::new(Stack::new());
        let err = |message: &str| {
            let error = error.err(message);
            log::error!("{:?}", &error);
            errors.push(error);
        };
        let pass = |message: &str, err: Error| {
            let error = error.pass_with(message, err);
            log::error!("{:?}", &error);
            errors.push(error);
        };
        let shape: std::sync::Arc<
            parking_lot::lock_api::RwLock<parking_lot::RawRwLock, DisplacementShape>,
        > = self.shape.clone();
        let scheduler = self.thread_pool.scheduler();
        for bound in self.bounds.iter() {
            // _true_ if the caller has requisted to exit.
            // Note that in this case the file may be partially filled.
            if self.exit.load(Ordering::SeqCst) {
                break;
            }
            let results = results.clone();
            let _errors = errors.clone();
            let _error = error.clone();
            let shape = Arc::clone(&shape);
            let bound = bound.clone();
            let center = match bound.center() {
                Some(center) => center,
                None => {
                    err("bound.center()");
                    continue;
                }
            };
            let step = self.level_step;
            let thread_name = format!(
                "BuildDisplacementBoundCache displacement_by_steps {:.3}",
                center
            );
            log::info!("{}.build | Starting thread {thread_name}", &self.dbg);
          //  println!("{}.build | Starting thread {thread_name}", &self.dbg);
            let handle = scheduler
                .spawn_named(thread_name, move || {
                    let guard = shape.read();
                    match guard.part(&bound) {
                        Ok(shape) => match shape {
                            Some(shape) => {
                                results.push((center, Some(shape.displacement_by_steps(step))))
                            }
                            None => results.push((center, None)),
                        },
                        Err(err) => {
                            let error = _error.pass_with(
                                format!("task center:{center} guard.part"),
                                err.to_string(),
                            );
                            log::error!("{}", error);
                            _errors.push(error);
                        }
                    }
                    Ok(())
                })
                .map_err(|err| {
                    error.pass_with(format!("spawn task bound:{:?}", bound), err.to_string())
                });
            match handle {
                Ok(task) => tasks.push_back(task),
                Err(err) => pass("task handle", err),
            };
        }
        for task in tasks {
            log::trace!("{}.build | join thread {}", &self.dbg, task.name());
         //   println!("{}.build | join thread {}", &self.dbg, task.name());
            if let Err(err) = task.join() {
                pass("task join", err);
            }
        }
        (results, errors)
    }
}
