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
/// Provides logic to calculate and store cache used by [super::DisplacementCache].
///
pub struct BuildDisplacementCache {
    dbg: Dbg,
    shape: Arc<RwLock<DisplacementShape>>,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    /// Draught in meters
    draught_min: f64,
    draught_max: f64,
    /// qnt draught steps for hull
    draught_step: f64,
    thread_pool: Arc<ThreadPool>,
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
            dbg: Dbg::new(parent, "BuildDisplacementCache"),
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
    /// Creates and starts worker for [DisplacementCache::calculate].
    /// results: [[heel, trim, draught, volume, vx, vy, vz, area, ax, ay, az, ix, iy, wx, wy]]
    pub fn build(self) -> (Vec<Vec<f64>>, Vec<Error>) {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: VecDeque<JoinHandle<_>> = VecDeque::new();
        let aabb_results = Arc::new(Stack::new());
        let draft_results = Arc::new(Stack::new());
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
            if self.exit.load(Ordering::SeqCst) {
                break 'draught;
            }
            {
                let aabb_results = aabb_results.clone();
                let shape = shape.clone();
                let thread_name =
                    format!("BuildDisplacementCache aabb {draught}");
                log::info!("{}.build | Starting thread {thread_name}", &self.dbg);
                //  println!("Starting thread {thread_name}");
                let handle = scheduler
                    .spawn(move || {
                        let guard = shape.read();
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
                    Ok(task) => tasks.push_back(task),
                    Err(err) => pass("task handle", err),
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
                    let shape = Arc::clone(&shape);
                    let thread_name =
                        format!("BuildDisplacementCache displacement {draught} {heel} {trim}");
                    log::info!("{}.build | Starting thread {thread_name}", &self.dbg);
                    //  println!("Starting thread {thread_name}");
                    let handle = scheduler
                        .spawn_named(thread_name, move || {
                            let guard = shape.read();
                            draft_results.push((
                                heel,
                                trim,
                                draught,
                                guard.displacement(heel, trim, draught),
                                guard.waterline_area(heel, trim, draught),
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
                        Ok(task) => tasks.push_back(task),
                        Err(err) => pass("task handle", err),
                    };
                }
            }
        }
        for task in tasks {
            log::info!("{}.build | join thread {}", &self.dbg, task.name());
            if let Err(err) = task.join() {
                pass("task join", err);
            }
        }
        let mut vec_aabb = Vec::new();
        while !aabb_results.is_empty() {
            if let Some((draught, data)) = aabb_results.pop() {
                match data {
                    Ok(data) => vec_aabb.push((draught, data)),
                    Err(err) => pass("aabb_results", err),
                }
            }
        }
        let mut vec_results = Vec::new();
        while !draft_results.is_empty() {
            if let Some((heel, trim, draught, volume, area, inertia)) = draft_results.pop() {
                if let Some((_, (l_x, l_y))) = vec_aabb.iter().find(|(wl_d, _)| *wl_d == draught) {
                    let (volume, v_center) = match volume {
                        Ok((volume, center)) => (volume, center),
                        Err(err) => {
                            pass("draft_results volume", err);
                            continue;
                        }
                    };
                    let (area, a_center) = match area {
                        Ok((area, center)) => (area, center),
                        Err(err) => {
                            pass("draft_results area", err);
                            continue;
                        }
                    };
                    let (i_x, i_y) = match inertia {
                        Ok((x, y)) => (x, y),
                        Err(err) => {
                            pass("draft_results inertia", err);
                            continue;
                        }
                    };
                    vec_results.push(vec![
                        heel,
                        trim,
                        draught,
                        volume,
                        v_center.x(),
                        v_center.y(),
                        v_center.z(),
                        area,
                        a_center.x(),
                        a_center.y(),
                        a_center.z(),
                        i_x,
                        i_y,
                        *l_x,
                        *l_y,
                    ]);
                } else {
                    pass(
                        "draft_results",
                        error.err(format!("no aabb for draught:{draught}")),
                    );
                }
            }
        }
        (vec_results, errors)
    }
}
