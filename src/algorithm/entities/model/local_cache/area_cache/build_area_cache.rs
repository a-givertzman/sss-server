use crate::algorithm::entities::model::Shape;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler},
};
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
///
/// Provides logic to calculate and store cache used by [super::AreaCache].
///
/// See [super::AreaCacheConf] for more details about the fields.
//

pub struct BuildAreaCache {
    dbg: Dbg,
    shape: Shape,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    scheduler: Scheduler,
    exit: Arc<AtomicBool>,
}
//
//
impl BuildAreaCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        shape: Shape,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "BuildAreaCache"),
            shape,
            trim_steps,
            draught_steps,
            scheduler,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [AreaCache::calculate].
    /// results: [[trim, draught, area, x, z]]
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let shape = Arc::new(RwLock::new(self.shape.clone()));
        'draught: for &draught in &self.draught_steps {
            for &trim in &self.trim_steps {
                // _true_ if the caller has requisted to exit.
                // Note that in this case the file may be partially filled.
                if self.exit.load(Ordering::SeqCst) {
                    break 'draught;
                }
                //  let dbg_ = self.dbg.clone();
                let results = results.clone();
                let shape = shape.clone();
                let handle = self
                    .scheduler
                    .spawn(move || {
                        let guard = shape.read().expect("Unable to read");
                        results.push((
                            trim,
                            draught,
                            guard.windage_area(trim, draught),
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
                    Err(err) => errors.push(Err(err)),
                };
            }
        }
        for task in tasks {
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(Err(error));
            }
        }
        while !results.is_empty() {
            if let Some((heel, trim, draught, volume, area, inertia)) = results.pop() {
                if let Some((_, (l_x, l_y))) = aabb.iter().find(|(wl_d, _)| *wl_d == draught) {
                    let (volume, vx, vy, vz) = match volume {
                        Ok((volume, x, y, z)) => (volume, x, y, z),
                        Err(err) => {
                            results.push(Err(error.pass_with("results volume", err)));
                            continue;
                        }
                    };
                    let (area, ax, ay, az) = match area {
                        Ok((area, x, y, z)) => (area, x, y, z),
                        Err(err) => {
                            results.push(Err(error.pass_with("draft_results area", err)));
                            continue;
                        }
                    };
                    let (i_x, i_y) = match inertia {
                        Ok((x, y)) => (x, y),
                        Err(err) => {
                            results.push(Err(error.pass_with("draft_results inertia", err)));
                            continue;
                        }
                    };
                    results.push(Ok(vec![
                        heel, trim, draught, volume, vx, vy, vz, area, ax, ay, az, i_x, i_y, *l_x,
                        *l_y,
                    ]));
                } else {
                    results.push(Err(error.err(format!("no aabb for draught:{draught}"))));
                }
            }
        }
        //   dbg!(&results);
        results
    }
}
