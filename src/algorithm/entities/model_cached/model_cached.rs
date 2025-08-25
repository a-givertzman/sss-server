use crate::{
    algorithm::{
        entities::{
            model_cached::{
                AreaCache, AreaShape, BoundedAreaCache, CompartmentCache, DamagedCompartmentCache, DisplacementCache, DisplacementShape, Shape
            }, Bounds, Moment, Position
        },
        eval::BalanceCtx,
    },
    kernel::types::{Arc, RwLock},
    ship_model::{query::BalanceQuery, reply::BoundArea},
};
use indexmap::IndexMap;
use log::*;
use nalgebra::{Point3, UnitQuaternion, UnitVector3, Vector3};
use parry3d_f64::{math::{UnitVector, Vector}, query::PointQuery, shape::HalfSpace};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler},
};
use std::path::PathBuf;
//use super::floating_position::FloatingPosition;
use super::{LocalCache, ModelCachedConf};

///
struct FloatingPositionResult {
    heel: f64,
    trim: f64,
    draught: f64,
    precision: f64,
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
    displacement_shapes: Vec<Arc<RwLock<DisplacementShape>>>,
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
        let mut displacement_shapes: Vec<Arc<RwLock<DisplacementShape>>> = Vec::new();
        let delta_pos = Some(conf.model_center_coord.clone());
        let displacement_shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
            &dbg,
            conf.model_dir.clone().join(PathBuf::from("hull.stl")),
            delta_pos,
            conf.model_scale,
        )));
        displacement_shapes.push(displacement_shape.clone());
        let windage_shape = Arc::new(RwLock::new(AreaShape::new_uninit(
            &dbg,
            conf.model_dir.clone().join(PathBuf::from("hull.stl")),
            Some(conf.model_dir.clone().join(PathBuf::from("additionals"))),
            delta_pos,
            conf.model_scale,
        )));
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
            .filter(|path| path.file_name().is_some())
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
                displacement_shapes.push(shape.clone());
                Some((
                    name.clone(),
                    CompartmentCache::new(
                        &dbg,
                        shape.clone(),
                        conf.cache_dir.clone().join(PathBuf::from("compartments")),
                        name,
                        conf.heel_steps.clone(),
                        conf.trim_steps.clone(),
                        conf.compartment_level_step,
                        scheduler.clone(),
                    ),
                ))
            })
            .flat_map(|v| v)
            .collect();
        let damaged_compartments = pathes
            .iter()
            .filter(|path| path.file_name().is_some())
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
                    delta_pos,
                    conf.model_scale,
                )));
                displacement_shapes.push(shape.clone());
                Some((
                    name.clone(),
                    DamagedCompartmentCache::new(
                        &dbg,
                        shape.clone(),
                        conf.cache_dir.clone().join(PathBuf::from("compartments")),
                        name,
                        conf.heel_steps.clone(),
                        conf.trim_steps.clone(),
                        conf.compartment_level_step,
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
                conf.hull_draught_step,
                scheduler.clone(),
            ),
            compartments,
            damaged_compartments,
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
        for shape in &self.displacement_shapes {
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
                    error.pass_with(format!("spawn task displacement_shape"), err.to_string())
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
            TODO
        dbg!("windage_area end");
        dbg!("bounded_windage_area start");
            TODO
        dbg!("bounded_windage_area end");
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
    pub fn rebuild_bounds(&self, bounds_qnt: usize) -> Result<Bounds, Error> {
        /*     TODO: rebuild
        model_bounded
        compartments_bounded
        bounded_windage_area
        */

        Ok(())
    }
    //
    pub fn rebuild_bounded_windage_area(&self, bounds: Bounds) -> Result<Vec<f64>, Error> {
        /*     TODO: 
        */
        Ok(())
    }
    //
    pub fn windage_area(&self) -> Result<(f64, f64), Error> {
        /*     TODO: 
        */
        Ok(())
    }
    //
    pub fn balance(&mut self, query: BalanceQuery) -> Result<BalanceCtx, Error> {
        let error = Error::new(&self.dbg, "balance");
        let FloatingPositionResult{roll, trim, draught_mid, volume, precision} = self
            .floating_position(query)
            .map_err(|err| error.pass_with("self.floating_position", err))?;

        let result = BalanceCtx {
            trim,
            draught_mid,
            roll,
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
    }
    //
    // (roll, trim, draught_mid, volume, precision)
    fn floating_position(
        &mut self,
        query: BalanceQuery,
    ) -> Result<FloatingPositionResult, Error> {
        let error = Error::new(&self.dbg, "eval");
        if query.water_density <= 0.  {
            return Err(error.err("water_density <= 0."));
        }
        let init_center = self.model_center_coord.clone();

        // постоянная масса
        let mass_const = query.mass_const; 
        // постоянный момент
        let moment_const = query.moment_const; 

        // Считаем сыпучие грузы.
        // На них крен и дифферент не влияет.
        let (mass_bulk, moment_bulk, bulk_result)  = {
            let mut vals = vec![None; 9];
            let result: Vec<_> = query.bulk.iter()
                .map(|v| {
                    vals[3] = Some(v.volume);
                    let (level, center) = match self.compartments.get(&v.space_id).map(|c| c.get(&vals)) {
                        Some(v) => match v {
                            Ok(v) => (v[2], Position::new(v[4], v[5], v[6])),
                            Err(err) => {
                                log::error!("{}", error.pass_with("bulk self.compartments.get", err));
                                (0., Moment::zero())
                            },
                        },
                        None => {
                            log::error!("{}", error.err(format!("bulk no compartment:{} in compartments", v.space_id)));
                            (0., Moment::zero())
                        },
                    };
                    (v.space_id.clone(), level, Moment::from_pos(center, v.mass))
            }).collect();    
            (
                query.bulk.iter().map(|v| v.mass).sum(), 
                result.iter().map(|(_, _, m)| *m).sum(),
                result
            )
        };

        let mass_liquid = query.liquid.iter().map(|v| v.mass).sum();
        let mut heel = 0.0;
        let mut trim = 0.0;
        let mut draught = self.draught_min;
        loop {
            let moment_liquid = {
                let mut vals = vec![None; 9];
                query.liquid.iter()
                    .map(|v| {
                        vals[3] = Some(v.volume);
                        let (level, center) = match self.compartments.get(&v.space_id).map(|c| c.get(&vals)) {
                            Some(v) => match v {
                                Ok(v) => (v[2], Position::new(v[4], v[5], v[6])),
                                Err(err) => {
                                    log::error!("{}", error.pass_with("liquid self.compartments.get", err));
                                    (0., Moment::zero())
                                },
                            },
                            None => {
                                log::error!("{}", error.err(format!("liquid no compartment:{} in compartments", v.space_id)));
                                (0., Moment::zero())
                            },
                        };
                        Moment::from_pos(center, v.mass)
                }).sum()
            };
            let (mass_damaged_compartment, moment_damaged_compartment)  = {
                let mut vals = vec![None; 9];
                query.damaged_compartment.iter()
                    .map(|v| {
                        vals[0] = Some(heel);
                        vals[1] = Some(trim);
                        vals[2] = Some(draught);
                        let (volume, center) = match self.compartments.get(v).map(|c| c.get(&vals)) {
                            Some(v) => match v {
                                Ok(v) => (v[3], Position::new(v[4], v[5], v[6])),
                                Err(err) => {
                                    log::error!("{}", error.pass_with("damaged_compartment self.compartments.get", err));
                                    (0., Moment::zero())
                                },
                            },
                            None => {
                                log::error!("{}", error.err(format!("damaged_compartment no compartment:{} in compartments", v)));
                                (0., Moment::zero())
                            },
                        };
                        let mass = volume*query.water_density;
                        (mass, Moment::from_pos(center, mass))
                }).fold((0., Moment::zero()), |(mass_sum, moment_sum), (mass, moment)| (
                    (mass_sum + mass, moment_sum + moment)
                ))
            };
            let mass_sum = mass_const + mass_bulk + mass_liquid + mass_damaged_compartment;
            // считаем корпус 
            let (draught, volume, volume_center) = {
                // Prepare values (key) to extract data from `self.cache`.
                // Note that 3rd parameter sets to None (as well as 5th and the rest).
                // This means we expect to get their approximated values from the cache.
                let mut vals = vec![None; 13];
                vals[0] = Some(heel);
                vals[1] = Some(trim);
                vals[3] = Some(mass_sum/query.water_density);
                // The cache returns the whole row(s) for given `vals`
                // and we expect each row has at least the following structure:
                //
                // [heel, trim, draught, volume, x, y, z, area, x, y, z, waterline_x, waterline_y]
                // where
                // - every value is of type f64,
                //
                let result = self.displacement
                    .get(&vals)
                    .map_err(|err| error.pass_with(format!("self.displacement.get vals:{:?}", vals), err))?;
                (result[2], result[3], Position::new(result[4], result[5], result[6])) 
            };

            let mass_center = {
                let moment_sum = moment_const + moment_bulk + moment_liquid + moment_damaged_compartment;
                moment_sum.to_pos(mass_sum)
            };

            // Определение невязки
            {
                let heel_rad = -heel.to_radians();
                let trim_rad = trim.to_radians();
                let trim_rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), trim_rad);
                let transformed_x_axis = trim_rotation.transform_vector(&Vector3::x_axis());
                let transformed_x_axis = UnitVector3::new_normalize(transformed_x_axis);
                let heel_rotation = UnitQuaternion::from_axis_angle(&transformed_x_axis, heel_rad);
                let rotation = heel_rotation * trim_rotation;
                let up_vector = rotation.transform_vector(&Vector3::z_axis());
                let up_vector = UnitVector::new_normalize(up_vector);
                // Через центр плавучести CB проводится горизонтальная плоскость                
                let my_plane = HalfSpace::new(up_vector);
                let cg = Point3::from_slice(&(volume_center - mass_center).values());

                let cg_h = my_plane.project_local_point(&cg, false).point;
                let l = (Position::new(cg_h.x, cg_h.y, cg_h.z) - Position::new(cg.x, cg.y, cg.z)).len();
                if l < query.precision {
                    return Ok(FloatingPositionResult {
                        heel,
                        trim,
                        draught,
                        precision: l,
                    });
                }
            }

            // Определение посадки судна для следующего шага
            let [frac_delta_psi_2, frac_delta_theta_2] = {
                let heel_rad = -heel.to_radians();
                let trim_rad = trim.to_radians();
                let trim_rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), trim_rad);
                let transformed_x_axis = trim_rotation.transform_vector(&Vector3::x_axis());
                let transformed_x_axis = UnitVector3::new_normalize(transformed_x_axis);
                let heel_rotation = UnitQuaternion::from_axis_angle(&transformed_x_axis, heel_rad);
                let rotation = heel_rotation * trim_rotation;
                let up_vector = rotation.transform_vector(&Vector3::z_axis());
                let up_vector = UnitVector::new_normalize(up_vector);
                // Через центр плавучести CB проводится горизонтальная плоскость                
                let my_plane = HalfSpace::new(up_vector);
                let cg = Point3::from_slice(&(volume_center - mass_center).values());

                let cg_h = my_plane.project_local_point(&cg, false);

                let cg = Point::from(self.disp_center.point());
                let size = self.centreline.len();
                let v_plane_normal = self.centreline.dir().cross(&Vector::unit_z());
                let v_plane =
                    Face::rect(&self.disp_center, &v_plane_normal, 0.5 * size, 1.5 * size);
                let frac_delta_psi_2 = 0.5 * {
                    let cb_v = v_plane
                        .project(&volume_center /* ~ CB */)
                        .map(|vertex| Point::from(vertex.point()))
                        .map_err(|err| {
                            error.pass_with("cb_v = m_plane.project", err.to_string())
                        })?;
                    let cg_h = {
                        let [.., z] = *cb_v;
                        let [x, y, ..] = self.disp_center.point();
                        Point::from(Vertex::<Attr>::new([x, y, z]).point())
                    };
                    //           ^
                    //           Z
                    //           |
                    //     +--+--+
                    //     |4 |1 |  \
                    //     +--#--+   > part of vertical plane, where # points to CG,
                    //     |3 |2 |  /  split into possible sections
                    // <---+--+--+
                    //  \
                    //   centreline direction projected on horizontal plane
                    //
                    // If _delta psi_ is located in 1 or 3 section (see pic),
                    // its value becomes positive (by defenition of trim).
                    //
                    // Example for section 3:
                    //    .... # - CG
                    //    .   /|
                    //    .  /=|-- delta psi
                    //    . /  |
                    //    ./   |
                    // <--#----# - CG_H
                    // |   \
                    // |    CB_V
                    // |
                    //  centreline direction projected on horizontal plane
                    //
                    let sign = {
                        let [section_1, section_3] = {
                            let [.., cg_z] = *cg;
                            let [cb_v_x, ..] = *cb_v;
                            let [cg_h_x, _, cg_h_z] = *cg_h;
                            [
                                cb_v_x >= cg_h_x && cg_h_z >= cg_z,
                                cb_v_x <= cg_h_x && cg_h_z <= cg_z,
                            ]
                        };
                        if section_1 || section_3 { 1.0 } else { -1.0 }
                    };
                    // calculate 'delta psi' (see schema in algorithm docs)
                    // taking into account its section (see `sign` comments)
                    sign * Vector::from([cg, cb_v]).angle(&Vector::from([cg, cg_h]))
                };
                let m_plane_normal = self.middle.normal_at(&self.middle.center());
                let frac_delta_theta_2 = 0.5 * {
                    let m_plane = {
                        let size = 0.5 * size;
                        Face::rect(&self.disp_center, &m_plane_normal, size, size)
                    };
                    let intersection = v_plane
                        .intersect(&m_plane, OpConf { parallel: true })
                        .edges()
                        .into_iter()
                        .next()
                        .as_ref()
                        .map(Edge::dir)
                        .ok_or(format!(
                            "{} | No intersection between Vertical\
                                plane and Parallel to Midlle planes.",
                            &self.dbg
                        ))?;
                    let cb_m = m_plane
                        .project(&volume_center)
                        .map(|vertex| Point::from(vertex.point()))
                        .map_err(|err| {
                            error.pass_with("cb_m = m_plane.project", err.to_string())
                        })?;
                    // Consider the tail of the model is behind the drawn part, then:
                    //      |
                    //   +--+--+
                    //   |4 |1 |  \
                    //   +--#--+   > part of plane parallel to `self.middle`,
                    //   |3 |2 |  /  where # points to CG, split into possible sections
                    // --+--+--+--
                    // |    |
                    // |     edge, which is the result of plane parallel to `self.middle`
                    // |     and _vertical_ plane intersection
                    // |
                    //  edge, which is the result of plane parallel to `self.middle`
                    //  and _horizontal_ plane intersection
                    //
                    // If _delta theta_ is located in 2 or 4 section (see pic),
                    // its value becomes negative (by defenition of heel).
                    //
                    // Example for section 2:
                    // CG - #.....
                    //      |\   .
                    //      |=\--.-- delta theta
                    //      |  \ .
                    //      |   \.
                    //    --#----#-- - edge, which is the result of plane parallel to `self.middle`
                    //     /    /      and _horizontal_ plane intersection
                    //    /     CB_M
                    //   /
                    //  point of vertical plane, horizontal plane,
                    //  and plane parallel to `self.middle` intersection
                    //
                    let sign = {
                        let [section_2, section_4] = {
                            let [.., cg_y, cg_z] = *cg;
                            let [.., cb_m_y, cb_m_z] = *cb_m;
                            [
                                cg_y >= cb_m_y && cg_z >= cb_m_z,
                                cg_y <= cb_m_y && cg_z <= cb_m_z,
                            ]
                        };
                        if section_2 || section_4 { -1.0 } else { 1.0 }
                    };
                    // calculate 'delta psi' (see schema in algorithm docs)
                    // taking into account the actual section (see `sign` comments)
                    sign * Vector::from([cg, cb_m]).angle(&intersection)
                };
                // apply rotations to `self.middle` and `self.centreline`
                {
                    let cg = Vertex::new(*cg);
                    self.middle = self
                        .middle
                        .rotate(cg.clone(), v_plane_normal, frac_delta_psi_2);
                    self.centreline = self
                        .centreline
                        .rotate(cg.clone(), v_plane_normal, frac_delta_psi_2)
                        .rotate(cg, m_plane_normal, frac_delta_theta_2);
                }
                [frac_delta_psi_2, frac_delta_theta_2]
            };
            // Align the current keel point vertiacally to its initial position.
            // This is necessary to get correct results from cache on next iterations.
            {
                let dir = {
                    let cur_keel = Point::from(self.centreline.center().point());
                    let new_keel = {
                        let [x, y, ..] = init_center;
                        let [.., z] = *cur_keel;
                        Point::from([x, y, z])
                    };
                    Vector::from([cur_keel, new_keel])
                };
                self.middle = self.middle.translate(dir);
                self.centreline = self.centreline.translate(dir);
            }
            heel += frac_delta_theta_2;
            trim += frac_delta_psi_2;
        }
    }
}
