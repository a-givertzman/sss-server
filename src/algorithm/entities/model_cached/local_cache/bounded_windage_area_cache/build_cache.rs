use crate::{algorithm::entities::model_cached::Shape, kernel::types::{Arc, RwLock}};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler},
};
use std::sync::atomic::{AtomicBool, Ordering};
///
/// Provides logic to calculate and store cache used by [super::BoundedAreaCache].
///
pub struct BuildBoundedAreaCache {
    dbg: Dbg,
    shape: Arc<RwLock<Shape>>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl BuildBoundedAreaCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        shape: Arc<RwLock<Shape>>,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "BuildAreaCache"),
            shape: shape.clone(),
            trim_steps,
            draught_steps,
            scheduler,
            exit,
        }
    }
    ///
    /// Creates and starts worker
    /// results: [[trim, draught, area, x]]
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let shape = self.shape.clone();
        'draught: for &draught in &self.draught_steps {
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
                        let guard = shape.read();
                        task_results.push((
                            trim,
                            draught,
                            guard.bounded_windage_area(trim, draught),
                        ));
                        Ok(())
                    })
                    .map_err(|err| {
                        error.pass_with(
                            format!("spawn task draught:{} trim:{}", draught, trim),
                            err.to_string(),
                        )
                    });
                match handle {
                    Ok(task) => tasks.push(task),
                    Err(err) => results.push(Err(err)),
                };
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
            if let Some((trim, draught, area)) = task_results.pop() {
                    let (_, _, mut values) = match area {
                        Ok((start_x, end_x, values)) => (start_x, end_x, values),
                        Err(err) => {
                            results.push(Err(error.pass_with("results area", err)));
                            continue;
                        }
                    };
                    let mut result = vec![trim, draught];
                    result.append(&mut values);
                    results.push(Ok(result));
            }
        }
        //   dbg!(&results);
        results
    }
}
