use crate::algorithm::entities::model::Shape;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler},
};
use std::sync::{
    atomic::{AtomicBool, Ordering}, Arc, RwLock
};
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
    /// results: [[heel, trim, draught, volume, x, y, z, area, x, y, z, waterline_x, waterline_y]]
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let aabb_results = Arc::new(Stack::new());
        let draft_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let shape = Arc::new(RwLock::new(self.shape.clone()));
        'draught: for &draught in &self.draught_steps {
            if self.exit.load(Ordering::SeqCst) {
                break 'draught;
            }
            {
                let aabb_results = aabb_results.clone();            
                let shape = shape.clone();
                let handle = self
                    .scheduler
                    .spawn(move || {
                        let guard = shape.read().expect("Unable to read");
                        aabb_results.push((draught, guard.aabb(draught)));
                        Ok(())
                    })
                    .map_err(|err| {
                        error.pass_with(
                            format!("spawn task aabb draught:{draught}"),
                            err.to_string(),
                        )
                    });
                match handle {
                    Ok(task) => tasks.push(task),
                    Err(err) => results.push(Err(err)),
                };
            }
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
                            let guard = shape.read().expect("Unable to read");
                            draft_results.push((
                                heel,
                                trim,
                                draught,
                                guard.displacement(heel, trim, draught),
                                guard.area(heel, trim, draught),
                                guard.inertia(heel, trim, draught),
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
        let mut aabb = Vec::new();
        while !aabb_results.is_empty() {
            if let Some((draught, data)) = aabb_results.pop() {
                match data {
                    Ok(data) => aabb.push((draught, data)),
                    Err(err) => results.push(Err(error.pass_with("aabb_results", err))),
                }
            }
        }
        while !draft_results.is_empty() {
            if let Some((heel, trim, draught, volume, area, inertia)) = draft_results.pop() {
                if let Some((_, (l_x, l_y))) = aabb.iter().find(|(wl_d, _)| *wl_d == draught) {
                    let (volume, vx, vy, vz) = match volume {
                        Ok((volume, x, y, z)) => (volume, x, y, z),
                        Err(err) => {
                            results.push(Err(error.pass_with("draft_results volume", err)));
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
                    results.push(Ok(vec!(heel, trim, draught, volume, vx, vy, vz, area, ax, ay, az, i_x, i_y, *l_x, *l_y)));
                } else {
                    results.push(Err(error.err(format!("no aabb for draught:{draught}"))));
                }
            }
        }
        //   dbg!(&results);
        results
    }
}
