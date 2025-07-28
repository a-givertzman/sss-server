use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{sync::Stack, thread_pool::{JoinHandle, Scheduler}};
use std::sync::{
    atomic::{AtomicBool, Ordering}, Arc, RwLock
};
use crate::algorithm::entities::model::Shape;
///
/// Provides logic to calculate and store cache used by [super::DisplacementCache].
///
/// See [super::DisplacementCacheConf] for more details about the fields.
//

pub struct BuildDisplacementCache {
    dbg: Dbg,
    shape: Shape,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl BuildDisplacementCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        shape: Shape,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "BuildDisplacementCache"),
            shape,
            heel_steps,
            trim_steps,
            draught_steps,
            scheduler,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [DisplacementCache::calculate].
    /// results: [[heel, trim, draught, volume, x, y, z]]
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        //  let mut waterline: Face<ShipModelMeta> = Workplane::xy().translated(origin).rect(&rect).to_face();
        let shape = Arc::new(RwLock::new(self.shape.clone()));
        'draught: for &draught in &self.draught_steps {
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        break 'draught;
                    }
                    //  let dbg_ = self.dbg.clone();
                    let task_results = task_results.clone();
                    let shape = shape.clone();
                    let handle = self
                        .scheduler
                        .spawn(move || {
                            let guard = shape.read().expect("Unable to read");
                            task_results.push(guard.displacement(heel, trim, draught));
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
        while !task_results.is_empty() {
            if let Some(data) = task_results.pop() {
                results.push(data);
            }
        }
        //   dbg!(&results);
        results
    }
}
