use nalgebra::OPoint;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler},
};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    algorithm::entities::{
        Position,
        model_cached::{DisplacementShape, Shape, draught},
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
    level_qnt_steps: usize,
    /// центр полного объема из бд
    center_max: Option<Position>,
    /// полный объем из бд
    volume_max: Option<f64>,
    scheduler: Scheduler,
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
        level_qnt_steps: usize,
        center_max: Option<Position>,
        volume_max: Option<f64>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            dbg: Dbg::new(parent, "BuildCompartmentCache"),
            shape: shape.clone(),
            heel_steps,
            trim_steps,
            level_qnt_steps,
            center_max,
            volume_max,
            scheduler,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [CompartmentCache::calculate].
    ///
    /// results: [[heel, trim, draught, volume, vx, vy, vz, ix, iy]]
    pub fn build(self) -> (Vec<Vec<f64>>, Vec<Error>) {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let draft_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let shape = self.shape.clone();
        let (volume_max, center_max) = match shape.read().properties() {
            Ok((volume_max, center_max)) => (volume_max, center_max),
            Err(err) => return (vec![], vec![error.pass_with("shape.properties", err)]),
        };
        let center_max = self.center_max.clone().unwrap_or(center_max);
        let volume_max = self.volume_max.clone().unwrap_or(volume_max);
        let draught_steps = match shape.read().draught_steps(self.level_qnt_steps) {
            Ok(draught_steps) => draught_steps,
            Err(err) => return (vec![], vec![error.pass_with("shape.read().draught_steps()", err)]),
        };
        let draught_zero = match shape.read().size() {
            Ok((_, _, _, z_min)) => z_min,
            Err(err) => return (vec![], vec![error.pass_with("shape.read().size()", err)]),
        };
        if draught_steps.len() < 2 {
            return (vec![], vec![error.err("draught_steps.len < 2")]);
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
                        Err(err) => errors.push(err),
                    };
                }
            }
        }
        for task in tasks {
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(error);
            }
        }
        while !draft_results.is_empty() {
            if let Some((heel, trim, draught, volume, inertia)) = draft_results.pop() {
                let (volume, center) = match volume {
                    Ok((volume, center)) => (volume, center),
                    Err(err) => {
                        errors.push(error.pass_with("draft_results volume", err));
                        continue;
                    }
                };
                let (i_x, i_y) = match inertia {
                    Ok((x, y)) => (x, y),
                    Err(err) => {
                        errors.push(error.pass_with("draft_results inertia", err));
                        continue;
                    }
                };
                if volume >= volume_max {
                    // для полного объема значения заполняем руками для нормальной интерполяции
                    results.push(vec![heel, trim, draught, volume_max, center_max.x(), center_max.y(), center_max.z(), 0., 0.]);
                } else {
                    results.push(vec![heel, trim, draught, volume, center.x(), center.y(), center.z(), i_x, i_y]);
                }                
            }
        }
        //   dbg!(&results);
        // для пустого объема значения заполняем руками для нормальной интерполяции
        // берем значения с ненулевым объемом
        let mut tmp: Vec<_> = results.iter().filter(|v| v[3] > 0.).collect();
        // и сортируем чтобы найти значение с минимальными креном/дифферентом и объемом
        tmp.sort_by(|a, b| (a[0].abs()*a[3] + a[1].abs()*a[3]).partial_cmp(&(b[0].abs()*b[3] + b[1].abs()*b[3])).unwrap());
        let center_min = if let Some(first) = tmp.first() {
            Position::new(first[4], first[5],draught_zero)
        } else {
            Position::new(center_max.x(), center_max.y(),draught_zero)
        };
        // для пустого объема значения меняем значения
        results.iter_mut().filter(|v| v[3] == 0.).for_each(|v| {
            v[4] = center_min.x();
            v[5] = center_min.y();
            v[6] = center_min.z();
            v[7] = 0.;
            v[8] = 0.;
        });
        //   dbg!(&results);
        (results, errors)
    }
/*
    pub fn build(self) -> Vec<Result<Vec<f64>, Error>> {
        log::info!("{}.build | Starting build", &self.dbg);
        let error = Error::new(&self.dbg, "build");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let draft_results = Arc::new(Stack::new());
        let mut results = Vec::new();
        let shape = self.shape.clone();
        let mut draught_steps = match shape.read().draught_steps(self.level_qnt_steps) {
            Ok(draught_steps) => draught_steps,
            Err(err) => return vec![Err(error.pass_with("shape.read().height()", err))],
        };
        if draught_steps.len() < 2 {
            return vec![Err(error.err("draught_steps.len < 2"))]
        }
        // для пустого объема значения заполняем руками для нормальной интерполяции
        {
            let draught_first = draught_steps.remove(0).max(0.);
            let heel_steps = self.heel_steps.clone();
            let trim_steps = self.trim_steps.clone();
            let draft_results = draft_results.clone();
            let shape = shape.clone();
            let error1 = error.clone();
            let error2 = error.clone();
            let handle = self
                .scheduler
                .spawn(move || {
                    let guard = shape.read();
                    let (x, y, _, _) = guard
                        .displacement(0., 0., draught_first)
                        .map_err(|err| error1.pass_with("guard.displacement for volume_max", err))?;
                    for &heel in &heel_steps {
                        for &trim in &trim_steps {
                            // для пустого отсека заполняем центр
                            draft_results.push((heel, trim, 0., Ok((0., x, y, 0.)), Ok((0., 0.))));
                        }
                    }
                    Ok(())
                })
                .map_err(|err| {
                    error2.pass_with("spawn task for empty compartment", err.to_string())
                });
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => results.push(Err(err)),
            };
        }
        // для полного объема значения заполняем руками для нормальной интерполяции
        {
            let draught_last = draught_steps.pop().unwrap();
            let heel_steps = self.heel_steps.clone();
            let trim_steps = self.trim_steps.clone();
            let draft_results = draft_results.clone();
            let center_max = self.center_max.clone();
            let volume_max = self.volume_max.clone();  
            let shape = shape.clone();      
            let error1 = error.clone();
            let error2 = error.clone();
            let handle = self
                .scheduler
                .spawn(move || {
                    let guard = shape.read();
                    let (x, y, z, volume) = match (center_max, volume_max) {
                        (Some(center), Some(volume)) => {
                            (center.x(), center.y(), center.z(), volume)
                        }
                        _ => {
                            let (volume, x, y, z) = guard
                                .displacement(0., 0., draught_last + 1.)
                                .map_err(|err| {
                                error1.pass_with("guard.displacement for volume_max", err)
                            })?;
                            (x, y, z, volume)
                        }
                    };
                    for &heel in &heel_steps {
                        for &trim in &trim_steps {
                            // для полного отсека объем не меняется от крена/дифферента и инерция всегда нулевая
                            draft_results.push((
                                heel,
                                trim,
                                draught_last,
                                Ok((volume, x, y, z)),
                                Ok((0., 0.)),
                            ));
                        }
                    }
                    Ok(())
                })
                .map_err(|err| {
                    error2.pass_with(format!("spawn task for full compartment"), err.to_string())
                });
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => results.push(Err(err)),
            };
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
        while !draft_results.is_empty() {
            if let Some((heel, trim, draught, volume, inertia)) = draft_results.pop() {
                let (volume, vx, vy, vz) = match volume {
                    Ok((volume, x, y, z)) => (volume, x, y, z),
                    Err(err) => {
                        results.push(Err(error.pass_with("draft_results volume", err)));
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
                results.push(Ok(vec![heel, trim, draught, volume, vx, vy, vz, i_x, i_y]));
            }
        }
        //   dbg!(&results);
        results
    }
    */
}
