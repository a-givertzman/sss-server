use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler},
};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    algorithm::entities::model_cached::{DisplacementShape, Shape},
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
    scheduler: Scheduler,
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
        scheduler: Scheduler,
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
            scheduler,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [CompartmentCache::calculate].
    /// 
    /// results: [[heel, trim, draught, volume, vx, vy, vz]]
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let draft_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let shape = self.shape.clone();
   /*     let draught_steps = match shape.read().draught_steps(0., self.draught_step) {
            Ok(draught_steps) => draught_steps,
            Err(err) => return vec![Err(error.pass_with("shape.read().height()", err))],
        };*/
        let mut draught_steps = Vec::new();
        let mut draught = self.draught_min;
        loop {
            draught_steps.push(draught);
            if draught >= self.draught_max {
                break;
            }
            draught += self.draught_step;
        } 
        'draught: for draught in draught_steps {
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        break 'draught;
                    }
                    //  let dbg_ = self.dbg.clone();
                    let draft_results = draft_results.clone();
                    let shape = shape.clone();
                    let handle = self
                        .scheduler
                        .spawn(move || {
                            let guard = shape.read();
                            draft_results.push((
                                heel,
                                trim,
                                draught,
                                guard.displacement(heel, trim, draught),
                            ));
                            Ok(())
                        })
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
                        Ok(task) => tasks.push(task),
                        Err(err) => results.push(Err(err)),
                    };
                }
            }
        }
        for task in tasks {
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                results.push(Err(error));
            }
        }
        while !draft_results.is_empty() {
            if let Some((heel, trim, draught, volume)) = draft_results.pop() {
                let (volume, vx, vy, vz) = match volume {
                    Ok((volume, x, y, z)) => (volume, x, y, z),
                    Err(err) => {
                        results.push(Err(error.pass_with("draft_results volume", err)));
                        continue;
                    }
                };
                results.push(Ok(vec![heel, trim, draught, volume, vx, vy, vz]));
            }
        }
        //   dbg!(&results);
        results
    }
}
