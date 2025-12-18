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
    algorithm::entities::model_cached::DisplacementShape,
    kernel::types::{Arc, RwLock},
};
///
/// Provides logic to calculate and store cache used by [super::DamagedCompartmentCache].
pub struct BuildDamagedCompartmentCache {
    dbg: Dbg,
    shape: Arc<RwLock<DisplacementShape>>,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_min: f64,
    draught_max: f64,
    draught_step: f64,
    thread_pool: Arc<ThreadPool>,
    exit: Arc<AtomicBool>,
}
//
//
impl BuildDamagedCompartmentCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_min: f64,
        draught_max: f64,
        draught_step: f64,
        thread_pool: Arc<ThreadPool>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        debug_assert!(draught_min < draught_max);
        debug_assert!(draught_step > 0.);
        Self {
            dbg: Dbg::new(parent, "BuildCompartmentCache"),
            shape: shape.clone(),
            heel_steps,
            trim_steps,
            draught_min,
            draught_max,
            draught_step,
            thread_pool,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [DamagedCompartmentCache::calculate].
    ///
    /// results: [[heel, trim, draught, volume, vx, vy, vz]]
    pub fn build(self) -> (Vec<Vec<f64>>, Vec<Error>) {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: VecDeque<JoinHandle<_>> = VecDeque::new();
        let results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let mut pass = |message: &str, err: Error| {
            let error = error.pass_with(message, err);
            log::error!("{:?}", &error);
            errors.push(error);
        };
        let shape = self.shape.clone();
        let mut draught_steps = Vec::new();
        let mut draught = self.draught_min;
        loop {
            draught_steps.push(draught);
            if draught >= self.draught_max {
                break;
            }
            draught += self.draught_step;
        }
        let scheduler = self.thread_pool.scheduler();
        'draught: for draught in draught_steps {
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        break 'draught;
                    }
                    let results = results.clone();
                    let shape = Arc::clone(&shape);
                    let thread_name =
                        format!("BuildDamagedCompartmentCache displacement {draught} {heel} {trim}");
                    log::info!("{}.build | Starting thread {thread_name}", &self.dbg);
                    //  println!("Starting thread {thread_name}");
                    let handle = scheduler
                        .spawn_named(
                            thread_name,
                            move || {
                                let guard = shape.read();
                                results.push((
                                    heel,
                                    trim,
                                    draught,
                                    guard.displacement(heel, trim, draught),
                                ));
                                Ok(())
                            },
                        )
                        .map_err(|err| {
                            error.pass_with(
                                format!(
                                    "spawn task draught:{} heel:{} trim:{}",
                                    draught, heel, trim
                                ),
                                err.to_string(),
                            )
                        });
                    match handle {
                        Ok(task) => tasks.push_back(task),
                        Err(err) => pass("task handle", err),
                    };
                }
            }
        }
        for task in tasks {
            log::trace!("{}.build | join thread {}", &self.dbg, task.name());
            if let Err(err) = task.join() {
                pass("task join", err);
            }
        }
        let mut vec_results = Vec::new();
        while !results.is_empty() {
            if let Some((heel, trim, draught, volume)) = results.pop() {
                let (volume, center) = match volume {
                    Ok((volume, center)) => (volume, center),
                    Err(err) => {
                        pass("results volume", err);
                        continue;
                    }
                };
                vec_results.push(vec![
                    heel,
                    trim,
                    draught,
                    volume,
                    center.x(),
                    center.y(),
                    center.z(),
                ]);
            }
        }
        //   dbg!(&results);
        (vec_results, errors)
    }
}
