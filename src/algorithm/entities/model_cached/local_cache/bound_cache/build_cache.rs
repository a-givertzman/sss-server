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
    draught_min: f64,
    draught_max: f64,
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
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let draft_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let shape = self.shape.clone();     
        for bound in self.bounds.iter() {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        break;
                    }
                    //  let dbg_ = self.dbg.clone();
                    let draft_results = draft_results.clone();
                    let shape = shape.clone();
                    let bound = bound.clone();
                    let center = match bound.center().ok_or(error.err("bound.center()")) {
                        Ok(center) => center,
                        Err(err) => {
                            log::error!("{:?}", &err.into());
                            results.push(Err(err));
                            continue;
                        }
                    }; 
                    let step = self.draught_step;
                    let handle = self
                        .scheduler
                        .spawn(move || {
                            let guard = shape.read();
                            let shape = guard.part(&bound);
                            if let Ok(shape) = guard.part(&bound) {
                                draft_results.push((
                                        center,
                                        shape.map(|shape| shape.displacement_by_steps(step)),
                                ));
                     /*           match shape {
                                    Some(shape) => draft_results.push((
                                        bound.center(),
                                        Some(shape.displacement_by_steps(step)),
                                    )),
                                    None => draft_results.push((
                                        bound.center(),
                                        None,
                                    )),
                                }*/                              
                            }
                            Ok(())
                        })
                        .map_err(|err| {
                            error.pass_with(
                                format!(
                                    "spawn task bound:{:?}",
                                    bound
                                ),
                                err.to_string(),
                            )
                        });
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => results.push(Err(err)),
                    };
        }
        for task in tasks {
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                results.push(Err(error));
            }
        }
        while !draft_results.is_empty() {
            if let Some((dx, result)) = draft_results.pop() {
                match result {
                    Some(result) => match result {
                        Ok(result) => results.push((dx, result)),
                        Err(err) => {
                            let error = error.pass_with("result", err.to_string());
                            log::error!("{}", error);
                            results.push((dx, Err(error)));
                        }
                    },
                    None => results.push(Ok((dx, None))),
                };
            }
        }
        //   dbg!(&results);
        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        results.into_iter().map(|(x, v)| v).collect()
    }
}
