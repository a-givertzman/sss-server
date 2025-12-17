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
    algorithm::entities::{
        Position,
        cache::Cache,
        model_cached::{CompartmentCacheResult, DisplacementShape, Shape},
    },
    kernel::types::{Arc, RwLock},
};
///
/// Provides logic to calculate and store cache used by [super::CompartmentCache].
pub struct BuildCompartmentCache {
    dbg: Dbg,
    shape: Arc<RwLock<DisplacementShape>>,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    level_step_qnt: usize,
    thread_pool: Arc<ThreadPool>,
    exit: Arc<AtomicBool>,
}
//
//
impl BuildCompartmentCache {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        parent: &Dbg,
        shape: Arc<RwLock<DisplacementShape>>,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        level_step_qnt: usize,
        thread_pool: Arc<ThreadPool>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "BuildCompartmentCache"),
            shape: shape.clone(),
            heel_steps,
            trim_steps,
            level_step_qnt,
            thread_pool,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [CompartmentCache::calculate].
    ///
    /// results: [[heel, trim, draught, volume, vx, vy, vz, ix, iy, max_moment, max_volume]]
    pub fn build(self) -> (Vec<Vec<f64>>, Vec<Error>) {
        //  dbg!("BuildCompartmentCache build begin");
        log::info!("{}.build | Starting build", &self.dbg);
        let mut tasks: VecDeque<JoinHandle<_>> = VecDeque::new();
        let error = Error::new(&self.dbg, "build");
        let results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let mut pass = |message: &str, err: Error| {
            let error = error.pass_with(message, err);
            log::error!("{:?}", &error);
            errors.push(error);
        };
        let shape = self.shape.clone();
        let (volume_max, center_max) = match shape.read().properties() {
            Ok((volume_max, center_max)) => (volume_max, center_max),
            Err(err) => return (vec![], vec![error.pass_with("shape.properties", err)]),
        };
        let max_heel = self
            .heel_steps
            .iter()
            .fold(0., |acc, v| if acc < v.abs() { v.abs() } else { acc });
        let max_trim = self
            .trim_steps
            .iter()
            .fold(0., |acc, v| if acc < v.abs() { v.abs() } else { acc });
        let draught_steps =
            match shape
                .read()
                .draught_steps(self.level_step_qnt, max_heel, max_trim)
            {
                Ok(draught_steps) => draught_steps,
                Err(err) => {
                    return (
                        vec![],
                        vec![error.pass_with("shape.read().draught_steps()", err)],
                    );
                }
            };
        let draught_zero = match shape.read().size() {
            Ok((_, _, _, z_min)) => z_min,
            Err(err) => return (vec![], vec![error.pass_with("shape.read().size()", err)]),
        };
        if draught_steps.len() < 2 {
            return (vec![], vec![error.err("draught_steps.len < 2")]);
        }
        let scheduler = self.thread_pool.scheduler();
        assert!(self.heel_steps.len() > 1);
        assert!(self.trim_steps.len() > 1);
        assert!(self.heel_steps.contains(&0.));
        assert!(self.trim_steps.contains(&0.));
        assert!(draught_steps.len() > 1);
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
                        format!("BuildCompartmentCache displacement {draught} {heel} {trim}");
                    log::info!("{}.build | Starting thread {thread_name}", &self.dbg);
                 //   println!("{}.build | Starting thread {thread_name}", &self.dbg);
                    let handle = scheduler
                        .spawn_named(thread_name, move || {
                            let guard = shape.read();
                            results.push((
                                heel,
                                trim,
                                draught,
                                guard.displacement(heel, trim, draught),
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
         //   println!("{}.build | join thread {}", &self.dbg, task.name());
            if let Err(err) = task.join() {
                pass("task join", err);
            }
        }
        let mut vec_results = Vec::new();
        while !results.is_empty() {
            if let Some((heel, trim, draught, volume, inertia)) = results.pop() {
                let (volume, center) = match volume {
                    Ok((volume, center)) => (volume, center),
                    Err(err) => {
                        pass("draft_results volume", err);
                        continue;
                    }
                };
                let (i_x, i_y) = if volume < volume_max {
                    match inertia {
                        Ok((x, y)) => (x, y),
                        Err(err) => {
                            pass("draft_results inertia", err);
                            continue;
                        }
                    }
                } else {
                    (0., 0.)
                };
                vec_results.push(vec![
                    heel,
                    trim,
                    draught,
                    volume,
                    center.x(),
                    center.y(),
                    center.z(),
                    i_x,
                    i_y,
                    0., // i_x max
                    0., // abs_moment
                ]);
            }
        }
        // для пустого объема значения заполняем руками для нормальной интерполяции
        {
            // берем значения с ненулевым объемом
            let mut tmp: Vec<_> = vec_results.iter().filter(|v| v[3] > 0.).collect();
            // и сортируем чтобы найти значение с минимальными креном/дифферентом и объемом
            tmp.sort_by(|a, b| {
                (a[0].abs() * a[3] + a[1].abs() * a[3])
                    .partial_cmp(&(b[0].abs() * b[3] + b[1].abs() * b[3]))
                    .unwrap()
            });
            let center_min = if let Some(first) = tmp.first() {
                Position::new(first[4], first[5], draught_zero)
            } else {
                Position::new(center_max.x(), center_max.y(), draught_zero)
            };
            // для пустого объема значения меняем значения
            vec_results.iter_mut().filter(|v| v[3] == 0.).for_each(|v| {
                v[4] = center_min.x();
                v[5] = center_min.y();
                v[6] = center_min.z();
                v[7] = 0.;
                v[8] = 0.;
                v[9] = 0.;
                v[10] = 0.;
            });
        }
        for &heel in &self.heel_steps {
            let sin_theta = heel.to_radians().sin();
            let cos_theta = heel.to_radians().cos();
            let mut current_vec: Vec<_> = vec_results
                .iter_mut()
                .filter(|v| v[0] == heel)
                .collect::<Vec<_>>();
           let max_inertia_trans_x = current_vec
                .iter()
                .map(|v| v[7])
                .max_by(|a, b| a.partial_cmp(&b).unwrap())
                .unwrap(); 
            current_vec.iter_mut().for_each(|v| {
                    v[9] = max_inertia_trans_x;
                    v[10] = (v[5] * cos_theta + v[6] * sin_theta) * v[3]; // абсолютный момент жидкости            
                }
            );
         
 /*            current_vec.iter_mut().for_each(|v| 
                    v[10] = (v[5] * cos_theta + v[6] * sin_theta) * v[3] // абсолютный момент жидкости            
            );
            let max_abs_moment = current_vec
                .iter()
                .map(|v| v[10])
                .max_by(|a, b| (a*heel.signum()).partial_cmp(&(b*heel.signum())).unwrap())
                .unwrap();            
      //      println!("adasd heel:{heel} {sin_theta} {cos_theta} {max_abs_moment} {max_inertia_trans_x}");  
            current_vec.iter_mut().for_each(|v| {
                v[9] = max_inertia_trans_x;
                v[11] = max_abs_moment;
            });*/
        }
        (vec_results, errors)
    }
}
