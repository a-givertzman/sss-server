use super::{LocalCache, ModelCachedConf};
use crate::{
    algorithm::{
        entities::{
            Bounds, Moment, Position,
            model_cached::{
                AreaShape, CompartmentCache, DamagedCompartmentCache, DisplacementCache,
                DisplacementShape, Shape, WindageArea,
            },
        },
        //      eval::BalanceCtx,
    },
    kernel::types::{Arc, RwLock},
    ship_model::query::BalanceQuery,
};
use indexmap::IndexMap;
use log::*;
use nalgebra::{Point3, UnitQuaternion, UnitVector3, Vector3};
use parry3d_f64::{
    math::{UnitVector, Vector},
    query::PointQuery,
    shape::HalfSpace,
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler},
};
use std::{collections::HashMap, path::PathBuf};

///
#[derive(Debug)]
pub struct FloatingPositionResult {
    heel: f64,
    trim: f64,
    draught_mid: f64,
    precision: f64,
    volume: f64,
}
///
/// See [sal_3dlib::props::Attributes] to get more details about what the attribute type is.
pub struct ModelCached {
    dbg: Dbg,
    /// 3d model initial position in 3D space (midel).
    pub model_center_coord: Position,
    /// Waterline coord Z in 3D space (midel) initial position.
    draught_min: f64,
    /// Privides access to structure of the 3D element
    displacement_shapes: IndexMap<String, Arc<RwLock<DisplacementShape>>>,
    windage_shape: Arc<RwLock<AreaShape>>,
    /// Provides a number of calculations:
    /// - cache for model, [heel, trim, draught, volume, x, y, z, area, x, y, z, l_x, l_y ]
    displacement: DisplacementCache,
    /// - cache for compartments, [index of compartments, [heel, trim, level, volume, x, y, z, i_x, i_y ]]
    compartments: IndexMap<String, CompartmentCache>,
    /// - cache for damaged compartments, [index of compartments, [heel, trim, level, volume, x, y, z ]]
    damaged_compartments: IndexMap<String, DamagedCompartmentCache>,
    /// - cache for bounds of model, [index of bound, [trim, draught, volume ]]
    //   model_bounded: IndexMap<usize, Vec<BoundCache>>,
    /// - cache for bounds of compartments,  [index of bound, TODO]
    //    compartments_bounded: IndexMap<usize, IndexMap<usize, IndexMap<usize, BoundCache>>>,
    /// - cache for windage area
    windage_area: WindageArea,
    scheduler: Scheduler,
}
//
//
impl ModelCached {
    ///
    /// Creates a new instance.
    pub fn new(parent: &Dbg, conf: ModelCachedConf, scheduler: Scheduler) -> Result<Self, Error> {
        let dbg = Dbg::new(parent, "ModelCached");
        let error = Error::new(&dbg, "new");
        let mut displacement_shapes: IndexMap<String, Arc<RwLock<DisplacementShape>>> =
            IndexMap::new();
        let delta_pos = Some(conf.model_center_coord.clone());
        let displacement_shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
            &dbg,
            conf.model_dir.clone().join(PathBuf::from("hull.stl")),
            delta_pos,
            conf.model_scale,
        )));
        displacement_shapes.insert("hull".to_owned(), displacement_shape.clone());
        let windage_shape = Arc::new(RwLock::new(AreaShape::new_uninit(
            &dbg,
            conf.model_dir.clone().join(PathBuf::from("hull.stl")),
            Some(conf.model_dir.clone().join(PathBuf::from("additionals"))),
            delta_pos,
            conf.model_scale,
        )));
        let windage_area = WindageArea::new(
            &dbg,
            windage_shape.clone(),
            conf.cache_dir.clone(),
            conf.draught_min,
        );
        let path = conf.model_dir.clone().join(PathBuf::from("compartments"));
        let dir = std::fs::read_dir(&path).map_err(|err| {
            error.pass_with(
                format!("read additional dir {:?}", path.to_str()),
                err.to_string(),
            )
        })?;
        let pathes: Vec<_> = dir
            .into_iter()
            .filter_map(|f| f.ok())
            .map(|f| f.path())
            .collect();
        // TODO: подумать, что делать с этими ошибками
        /*    let (result, _errors): (Vec<_>, Vec<_>) = pathes
                    .into_iter()
                    .map(|p| (p.file_name(), p))
                    .partition(|(s, r)| s.is_some());
        */
        let compartments = pathes
            .iter()
            .filter(|path: &&PathBuf| path.file_name().is_some())
            .map(|path| {
                let Some(name) = path.file_stem() else {
                    return None;
                };
                let Some(name) = name.to_str() else {
                    return None;
                };
                let name = name.to_string();
                let shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
                    &dbg,
                    path.clone(),
                    None,
                    conf.model_scale,
                )));
                displacement_shapes.insert(name.clone(), shape.clone());
                let (volume_max, center_max) =
                    if let Some((volume_max, center_max)) = conf.compartment_data.get(&name) {
                        (*volume_max, *center_max)
                    } else {
                        (None, None)
                    };
                Some((
                    name.clone(),
                    CompartmentCache::new(
                        &dbg,
                        shape.clone(),
                        conf.cache_dir.clone().join(PathBuf::from("compartments")),
                        name.clone(),
                        conf.heel_steps.clone(),
                        conf.trim_steps.clone(),
                        conf.compartment_qnt_steps,
                        center_max,
                        volume_max,
                        scheduler.clone(),
                    ),
                ))
            })
            .flat_map(|v| v)
            .collect();
        let damaged_compartments = pathes
            .iter()
            .filter(|path: &&PathBuf| path.file_name().is_some())
            .map(|path| {
                let Some(name) = path.file_stem() else {
                    return None;
                };
                let Some(name) = name.to_str() else {
                    return None;
                };
                let name = name.to_string();
                let shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
                    &dbg,
                    path.clone(),
                    Some(conf.model_center_coord.clone()),
                    conf.model_scale,
                )));
                displacement_shapes.insert(name.clone() + "_damaged", shape.clone());
                Some((
                    name.clone(),
                    DamagedCompartmentCache::new(
                        &dbg,
                        shape.clone(),
                        conf.cache_dir
                            .clone()
                            .join(PathBuf::from("damaged_compartments")),
                        name.clone(),
                        conf.heel_steps.clone(),
                        conf.trim_steps.clone(),
                        conf.draught_min,
                        conf.draught_max,
                        conf.hull_draught_step,
                        scheduler.clone(),
                    ),
                ))
            })
            .flat_map(|v| v)
            .collect();
        let model_cached = Self {
            dbg: dbg.clone(),
            model_center_coord: conf.model_center_coord.clone(),
            draught_min: conf.draught_min,
            displacement_shapes,
            windage_shape,
            displacement: DisplacementCache::new(
                &dbg,
                displacement_shape.clone(),
                conf.cache_dir.clone(),
                conf.heel_steps.clone(),
                conf.trim_steps.clone(),
                conf.draught_min,
                conf.draught_max,
                conf.hull_draught_step,
                scheduler.clone(),
            ),
            compartments,
            damaged_compartments,
            windage_area,
            scheduler: scheduler.clone(),
        };
        Ok(model_cached)
    }
    ///
    ///
    /// Generates and reload the internal caches.
    ///
    /// The field `caches` contains cache keys to update.
    /// Remaining it empty builds and reloads all the caches.
    ///
    /// Note that it may take some time to complete
    /// due to the size of datasets and algorithm complexity.
    ///
    /// # Errors
    /// Internally it creates worker threads while building.
    /// The result error is a collection of all failed worker errors joined by '\n'.
    pub fn rebuild_caches(&mut self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "rebuild_caches");
        let mut errors = Vec::new();
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut results: Vec<Result<(), Error>> = Vec::new();
        // Сначала считаем модели в разных потоках
        dbg!("shapes start");
        for (name, shape) in &self.displacement_shapes {
            let shape = shape.clone();
            let task_results = task_results.clone();
            let handle = self
                .scheduler
                .spawn(move || {
                    let mut guard = shape.write();
                    task_results.push(guard.init());
                    Ok(())
                })
                .map_err(|err| {
                    error.pass_with(
                        format!("spawn task displacement_shape {name}"),
                        err.to_string(),
                    )
                });
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => results.push(Err(err)),
            };
        }
        {
            let shape = self.windage_shape.clone();
            let task_results = task_results.clone();
            let handle = self
                .scheduler
                .spawn(move || {
                    let mut guard = shape.write();
                    task_results.push(guard.init());
                    Ok(())
                })
                .map_err(|err| error.pass_with(format!("spawn task area_shape"), err.to_string()));
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => results.push(Err(err)),
            };
        }
        for task in tasks {
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                results.push(Err(error));
            }
        }
        dbg!("shapes end");
        dbg!("displacement start");
        // Считаем кэши, они сами по себе многопоточны, поэтому делить на потоки нет смысла
        if let Err(error) = self.displacement.rebuild() {
            errors.push(("displacement".to_owned(), error));
        }
        dbg!("displacement end");
        dbg!("windage_area start");
        if let Err(error) = self.windage_area.rebuild() {
            errors.push(("displacement".to_owned(), error));
        }
        dbg!("windage_area end");
        dbg!("compartments start");
        for (name, compartment) in &mut self.compartments {
            if let Err(error) = compartment.rebuild() {
                errors.push((("compartment ".to_owned() + name), error));
            }
        }
        dbg!("compartments end");
        dbg!("damaged_compartments start");
        for (name, compartment) in &mut self.damaged_compartments {
            if let Err(error) = compartment.rebuild() {
                errors.push((("damaged_compartment ".to_owned() + name), error));
            }
        }
        dbg!("damaged_compartments end");
        if !errors.is_empty() {
            return Err(error.pass_with(
                "rebuild_caches",
                errors.iter().fold(String::new(), |acc, (key, err)| {
                    format!("{acc}\n\tIn cache {:?} was error: {err}", key)
                }),
            ));
        }
        Ok(())
    }
    //
    /*   pub fn rebuild_bounds(&self, bounds_qnt: usize) -> Result<Bounds, Error> {

           let bounds =
           /*     TODO: rebuild
           model_bounded
           compartments_bounded
           bounded_windage_area
           */

           Ok(())
       }
    */
    //
    pub fn bounded_windage_area(&mut self, bounds: Bounds) -> Result<Vec<f64>, Error> {
        self.windage_area
            .bounded_windage_area(bounds)
            .map_err(|err| {
                Error::new(&self.dbg, "bounded_windage_area")
                    .pass_with("self.windage_area.bounded_windage_area", err)
            })
    }
    //
    pub fn windage_area(&mut self) -> Result<(f64, f64), Error> {
        self.windage_area.windage_area().map_err(|err| {
            Error::new(&self.dbg, "windage_area").pass_with("self.windage_area.windage_area", err)
        })
    }

    /*
    //
    pub fn balance(&mut self, query: BalanceQuery) -> Result<BalanceCtx, Error> {
        let error = Error::new(&self.dbg, "balance");
        let FloatingPositionResult {
            heel,
            trim,
            draught_mid,
            precision,
            volume,
        } = self
            .floating_position(query)
            .map_err(|err| error.pass_with("self.floating_position", err))?;

        let result = BalanceCtx {
            roll: heel,
            trim,
            draught_mid,
            bounds: todo!(),
            bulk: todo!(),
            liquid: todo!(),
            bounds_volume: todo!(),
            volume,
            area_wl: todo!(),
            length_wl: todo!(),
            breadth_wl: todo!(),
            volume_shift_z: todo!(),
            entry_angle: todo!(),
            flooding_angle: todo!(),
            bow_area: todo!(),
            const_area_v: todo!(),
            const_area_h: todo!(),
            rad_long: todo!(),
            rad_trans: todo!(),
            pantocaren: todo!(),
        };
    }    */
    /// Расчет равновесного положения
    pub fn floating_position(
        &mut self,
        query: BalanceQuery,
    ) -> Result<FloatingPositionResult, Error> {
        dbg!("floating_position start");
        let error = Error::new(&self.dbg, "eval");
        if query.water_density <= 0. {
            return Err(error.err("water_density <= 0."));
        }
        // постоянная масса
        let mass_const = query.mass_const;
        // постоянный момент
        let moment_const = query.moment_const;
        // Считаем сыпучие грузы.
        // На них крен и дифферент не влияет.
        dbg!("floating_position bulk start");
        let (mass_bulk, moment_bulk, bulk_result) = {
            let result: Vec<_> = query
                .bulk
                .iter()
                .map(|v| {
                    let (level, position) = 
                        match self.compartments.get(&v.space_id).map(|c| c.get(0., 0., v.volume, query.precision)) {
                            Some(v) => match v {
                                Ok((draught, position)) => (draught, position),
                                Err(err) => {
                                    log::error!(
                                        "{}",
                                        error.pass_with("bulk self.compartments.get", err)
                                    );
                                    (0., Moment::zero())
                                }
                            },
                            None => {
                                log::error!(
                                    "{}",
                                    error.err(format!(
                                        "bulk no compartment:{} in compartments",
                                        v.space_id
                                    ))
                                );
                                (0., Moment::zero())
                            }
                        };
                    (v.space_id.clone(), level, Moment::from_pos(position, v.mass))
                })
                .collect();
            (
                query.bulk.iter().map(|v| v.mass).sum::<f64>(),
                result.iter().map(|(_, _, m)| *m).sum(),
                result,
            )
        };
        dbg!("floating_position bulk end");

        dbg!("floating_position mass_liquid start");
        let mass_liquid = query.liquid.iter().map(|v| v.mass).sum::<f64>();
        let mut heel = 0.0;
        let mut trim = 0.0;
        let mut draught = self.draught_min;
        loop {
            let moment_liquid = {
                query
                    .liquid
                    .iter()
                    .map(|v| {
                        let (level, center) = 
                            match self.compartments.get(&v.space_id).map(|c| c.get(heel, trim, v.volume, query.precision)) {
                                Some(v) => match v {
                                    Ok((level, position)) => (level, position),
                                    Err(err) => {
                                        log::error!(
                                            "{}",
                                            error.pass_with("liquid self.compartments.get", err)
                                        );
                                        (0., Moment::zero())
                                    }
                                },
                                None => {
                                    log::error!(
                                        "{}",
                                        error.err(format!(
                                            "liquid no compartment:{} in compartments",
                                            v.space_id
                                        ))
                                    );
                                    (0., Moment::zero())
                                }
                            };
                        Moment::from_pos(center, v.mass)
                    })
                    .sum()
            };
            dbg!("floating_position mass_liquid end");
            dbg!("floating_position mass_damaged_compartment start");
            let (mass_damaged_compartment, moment_damaged_compartment) = {
                query
                    .damaged_compartment
                    .iter()
                    .map(|v| {
                        let (volume, center) =
                            match self.damaged_compartments.get(v).map(|c| c.get(heel, trim, draught)) {
                                Some(v) => match v {
                                    Ok((volume, center)) => (volume, center),
                                    Err(err) => {
                                        log::error!(
                                            "{}",
                                            error.pass_with(
                                                "damaged_compartment self.compartments.get",
                                                err
                                            )
                                        );
                                        (0., Moment::zero())
                                    }
                                },
                                None => {
                                    log::error!(
                                        "{}",
                                        error.err(format!(
                                            "damaged_compartment no compartment:{} in compartments",
                                            v
                                        ))
                                    );
                                    (0., Moment::zero())
                                }
                            };
                        let mass = volume * query.water_density;
                        (mass, Moment::from_pos(center, mass))
                    })
                    .fold(
                        (0., Moment::zero()),
                        |(mass_sum, moment_sum), (mass, moment)| {
                            (mass_sum + mass, moment_sum + moment)
                        },
                    )
            };
            dbg!("floating_position mass_damaged_compartment end");

            let mass_sum = mass_const + mass_bulk + mass_liquid + mass_damaged_compartment;
            let volume = mass_sum / query.water_density;
            // считаем корпус
            dbg!("floating_position hull start");
            let (new_draught, cb) =
                self.displacement.get(heel, trim, mass_sum / query.water_density, query.precision)
                .map_err(|err| error.pass_with(format!("self.displacement.get heel:{heel} trim:{trim} volume:{volume}"), err))?;

            dbg!("floating_position hull end");
            // расчет ориентации корпуса
            let rotation = {
                let heel_rad = -heel.to_radians();
                let trim_rad = trim.to_radians();
                let trim_rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), trim_rad);
                let transformed_x_axis = trim_rotation.transform_vector(&Vector3::x_axis());
                let transformed_x_axis = UnitVector3::new_normalize(transformed_x_axis);
                let heel_rotation = UnitQuaternion::from_axis_angle(&transformed_x_axis, heel_rad);
                heel_rotation * trim_rotation
            };
            // центр тяжести корпуса
            let cg = {
                let moment_sum =
                    moment_const + moment_bulk + moment_liquid + moment_damaged_compartment;
                moment_sum.to_pos(mass_sum) + self.model_center_coord
            };
            // Определение невязки
            let cg_h = {
                let up_vector = rotation.transform_vector(&Vector3::z_axis());
                let up_vector = UnitVector::new_normalize(up_vector);
                // Через центр плавучести CB проводится горизонтальная плоскость
                let my_plane = HalfSpace::new(up_vector);
                let cg_local = cg - cb;
                let cg_h = Position::from(my_plane.project_local_point(&cg_local.into(), false).point);
          //      dbg!(cg, cb, cg_local, cg_h);
                let precision = (cg_h - cg_local).len();
       //         dbg!(self.model_center_coord, cb, cg, cg_h, heel, trim, new_draught, precision);
                if precision < query.precision {
                    return Ok(FloatingPositionResult {
                        heel,
                        trim,
                        draught_mid: new_draught,
                        precision,
                        volume,
                    });
                }
                cb + cg_h
            };
    //        dbg!(cg, cb, cg_h);
            // Определение посадки судна для следующего шага
            let cb_v = {
                let up_vector = rotation.transform_vector(&Vector3::y_axis());
                let up_vector = UnitVector::new_normalize(up_vector);
                // Через центр плавучести CG проводится вертикальная плоскость параллельная основной линии
                let my_plane = HalfSpace::new(up_vector);
                let cb_local = cb - cg;
                let cb_v = Position::from(my_plane.project_local_point(&cb_local.into(), false).point);
         //       dbg!(cg, cb, cb_local, cb_v);
                cg + cb_v
            };
     //       dbg!(cb_v);
            let cb_m = {
                let up_vector = rotation.transform_vector(&Vector3::x_axis());
                let up_vector = UnitVector::new_normalize(up_vector);
                // Через центр плавучести CG проводится вертикальная плоскость параллельная миделю
                let my_plane = HalfSpace::new(up_vector);
                let cb_local = cb - cg;
                let cb_m = Position::from(my_plane.project_local_point(&cb_local.into(), false).point);
                cg + cb_m
            };
      //      dbg!(cb_m);
            // проекция точки cg_m на вертикальную плоскость параллельную основной линии
            let cg_m_h = {
                let up_vector = rotation.transform_vector(&Vector3::y_axis());
                let up_vector = UnitVector::new_normalize(up_vector);
                // Через центр плавучести CG проводится вертикальная плоскость параллельная основной линии
                let my_plane = HalfSpace::new(up_vector);
                let cg_m = cb_m - cg;
                let cg_m_h =
                    Position::from(my_plane.project_local_point(&cg_m.into(), false).point);
                cg + cg_m_h
            };
      //      dbg!(cg_m_h);

            let d_gh = (cg_h - cg).len();
            let d_bv = (cb_v - cg).len();
            let frac_delta_psi = (d_gh / d_bv).acos().to_degrees();
            let d_bm = (cb_m - cg).len();
            let d_gmh = (cg_m_h - cg).len();
            let frac_delta_theta = (d_gmh/ d_bm).acos().to_degrees();
            heel += frac_delta_theta / 2.;
            trim += frac_delta_psi / 2.;
            draught = new_draught;
            println!("cg:{cg} cg_h:{cg_h} cb:{cb} cb_v:{cb_v} cb_m:{cb_m} cg_m_h:{cg_m_h} d_gh:{d_gh} d_bv:{d_bv} psi:{frac_delta_psi} d_bm:{d_bm} d_gmh:{d_gmh} theta:{frac_delta_theta} heel:{heel} trim:{trim} draught:{draught}");
            dbg!("floating_position end");
        }
    }
}
