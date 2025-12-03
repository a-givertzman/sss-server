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
        model_cached::{DisplacementShape, Shape},
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
                    println!("{}.build | Starting thread {thread_name}", &self.dbg);
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
            println!("{}.build | join thread {}", &self.dbg, task.name());
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
                let (i_x, i_y) = match inertia {
                    Ok((x, y)) => (x, y),
                    Err(err) => {
                        pass("draft_results inertia", err);
                        continue;
                    }
                };
                if volume >= volume_max {
                    // для полного объема значения заполняем руками для нормальной интерполяции
                    vec_results.push(vec![
                        heel,
                        trim,
                        draught,
                        volume_max,
                        center_max.x(),
                        center_max.y(),
                        center_max.z(),
                        0.,
                        0.,
                        volume_max * center_max.y(),
                        0.,
                        0.,
                        0.,
                        0.,
                    ]);
                } else {
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
                        volume * center.y(),
                        0.,
                        0.,
                        0.,
                        0.,
                    ]);
                }
            }
        }
        // для пустого объема значения заполняем руками для нормальной интерполяции
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
        });
        // кэш значений при нулевых крене и дифференте для нахождения базового момента объема
        let mut base = vec_results
            .iter()
            .filter(|v| v[0] == 0. && v[1] == 0.)
            .map(|v| vec![v[3], v[3] * v[5]])
            .collect::<Vec<_>>();
        base.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        base.dedup();
     //   dbg!(&base);        
        let volume_cache = Cache::new(&self.dbg);
        volume_cache.init(base).unwrap(); //TODO err
        // Находим максимальный момент и соответствующий ему объем для каждого крена.
        // Знак момента соответствует стороне крена
        for &heel in &self.heel_steps {
            let mut current_vec: Vec<_> = vec_results
                .iter_mut()
                .filter(|v| v[0] == heel)
                .collect::<Vec<_>>();
            // считаем моменты и дельту
            let moments = current_vec
                .iter()
                .map(|v| {
                    let volume = v[3];
                    let moment = v[9];
                    let volume_shift = (v[3], v[4], v[5]);
                    let base_moment = volume_cache.get(&[volume])[0];
                    let delta_moment = moment - base_moment;
                    (delta_moment, moment, volume, volume_shift)
                })
                .collect::<Vec<_>>();
            let (_, moment_max, volume_from_moment, volume_shift) = if heel < 0. {
                moments.iter().max_by(|a, b| b.0.partial_cmp(&a.0).unwrap())
            } else {
                moments.iter().max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            }
            .unwrap_or(&(0., 0., 0., (0., 0., 0.))); // TODO err        
        //    println!("heel:{heel} {} {};", moment_max, volume_from_moment);
            // Каждому крену соответсвует максимальный момент и соответствующий ему объем
            current_vec.iter_mut().for_each(|v| {
                v[9] = *moment_max;
                v[10] = *volume_from_moment;
                v[11] = volume_shift.0;
                v[12] = volume_shift.1;
                v[13] = volume_shift.2;
            });
        }
        (vec_results, errors)
    }
}
