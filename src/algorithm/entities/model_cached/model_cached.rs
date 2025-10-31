use super::{LocalCache, ModelCachedConf};
use crate::{
    algorithm::{
        entities::{
            AddVec, Bounds, Moment, Position,
            model_cached::{
                AreaShape, BoundDisplacementCache, CompartmentCache, CompartmentCacheResult,
                DamagedCompartmentCache, DisplacementCache, DisplacementCacheResult,
                DisplacementShape, Draught, Shape, WindageArea,
            },
            ship_model::{stability_result::BalanceStabilityResult, *},
        },
        eval::{StrengthBalanceCtx, strength_balance_eval},
    },
    kernel::types::{Arc, RwLock},
};
use core::f64;
use indexmap::IndexMap;
use nalgebra::{UnitQuaternion, UnitVector3, Vector3};
use parry3d_f64::{query::PointQuery, shape::HalfSpace};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, ThreadPool},
};
use std::{collections::HashMap, fmt::Display, path::PathBuf};

/// Структура для ввода данных расчета равновесного положения корпуса судна.
#[derive(Debug, Clone)]
pub(crate) struct FloatingPositionQuery {
    // TODO - переименовать, тут лежат общие данные по судну - груз и т.п.
    /// Плотность забортной воды
    pub water_density: f64,
    /// масса судна порожнем и грузов размещенных на судне:
    /// генерального груза (unitCargoAssignment), контейнеров (containerCargoAssignment),
    /// газообразного груза (gaseousCargoAssignment), массы обледенения и намокания;
    pub mass_const: f64,
    /// Сумарный момент за вычетом смещяемых и насыпных грузов
    pub moment_const: Moment,
    /// навалочный груз
    pub bulk: Vec<BulkData>,
    /// жидкий груз
    pub liquid: Vec<LiquidData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>, // TODO
    /// номера поврежденных помещений
    pub damaged_compartment: Vec<String>,
    /// точность расчета
    pub epsilon: f64,
}
/// Результат расчета равновесного положения корпуса судна
#[derive(Debug)]
pub(crate) struct FloatingPositionResult {
    /// Крен, градусы
    pub heel: f64,
    /// Дифферент, градусы
    pub trim: f64,
    /// Осадка на миделе
    pub draught_mid: f64,
    /// Точность - расстояние в горизонтальной плоскости между центрами масс и водоизмещения
    pub precision: f64,
    /// Объемное водоизмещение, м^3
    pub displacement: f64,
    /// Смещение центра объемного водоизмещения, м
    pub displacement_center: Position,
    /// Площадь ватерлинии, м^2
    pub area_wl: f64,
    /// Смещение центра тяжести ватеринии, м
    pub area_wl_center: Position,
    /// Длинна по ватерлинии при текущей осадке, м
    pub length_wl: f64,
    ///  Ширина по ватерлинии при текущей осадке, м
    pub breadth_wl: f64,
    /// Продольный метацентрический радиус, м
    pub rad_long: f64,
    /// Поперечный метацентрические радиус, м
    pub rad_trans: f64,
}
//
impl Display for FloatingPositionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(heel:{:.6}, trim:{:.6}, draught_mid:{:.6}, precision:{:.6},
            displacement:{:.6}, displacement_center:({:.6}, {:.6}, {:.6}), 
            area_wl:{:.6}, area_wl_center:({:.6}, {:.6}, {:.6}),  
            length_wl:{:.6}, breadth_wl:{:.6},
            rad_long:{:.6}, rad_trans:{:.6})",
            self.heel,
            self.trim,
            self.draught_mid,
            self.precision,
            self.displacement,
            self.displacement_center.x(),
            self.displacement_center.y(),
            self.displacement_center.z(),
            self.area_wl,
            self.area_wl_center.x(),
            self.area_wl_center.y(),
            self.area_wl_center.z(),
            self.length_wl,
            self.breadth_wl,
            self.rad_long,
            self.rad_trans,
        )
    }
}
///
/// See [sal_3dlib::props::Attributes] to get more details about what the attribute type is.
pub struct ModelCached {
    dbg: Dbg,
    /// Ship length between perpendiculars
    ship_length_lbp: f64,
    /// 3d model initial position in 3D space (midel).
    pub model_center_coord: Position,
    /// Waterline coord Z in 3D space (midel) initial position.
    draught_min: f64,
    /// Draught step for hull
    hull_draught_step: f64,
    /// Level step for bounds
    bounds_level_step: f64,
    /// Directory containing [super::ModelCached] caches.
    cache_dir: PathBuf,
    /// Privides access to structure of the 3D element
    displacement_shapes: IndexMap<String, Arc<RwLock<DisplacementShape>>>,
    windage_shape: Arc<RwLock<AreaShape>>,
    /// Provides a number of calculations:
    /// - cache for model, [heel, trim, draught, volume, x, y, z, area, x, y, z, l_x, l_y ]
    displacement: DisplacementCache,
    /// - cache for compartments, [index of compartments, [heel, trim, level, volume, x, y, z, i_x, i_y ]]
    compartments: IndexMap<String, Arc<RwLock<CompartmentCache>>>,
    /// - cache for damaged compartments, [index of compartments, [heel, trim, draught, volume, x, y, z ]]
    damaged_compartments: IndexMap<String, Arc<RwLock<DamagedCompartmentCache>>>,
    /// - cache for windage area
    windage_area: WindageArea,
    /// - cache for bounds of model, [qnt_bounds, cache]
    displacement_bounded: HashMap<usize, Arc<RwLock<BoundDisplacementCache>>>,
    /// - cache for bounds of compartments, [qnt_bounds, [compartment_id, cache]]
    compartments_bounded: HashMap<usize, IndexMap<String, Arc<RwLock<BoundDisplacementCache>>>>,
    thread_pool: Arc<ThreadPool>,
}
//
//
impl ModelCached {
    ///
    /// Creates a new instance.
    pub fn new(
        parent: &Dbg,
        conf: ModelCachedConf,
        thread_pool: Arc<ThreadPool>,
    ) -> Result<Self, Error> {
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
        let pathes: Vec<_> = match std::fs::read_dir(&path) {
            Ok(dir) => dir
                .into_iter()
                .filter_map(|f| f.ok())
                .map(|f| f.path())
                .collect(),
            Err(err) => {
                // TODO: подумать, что делать при неправильном имени файла
                log::error!(
                    "{}",
                    error.pass_with(
                        format!("read additional dir {:?}", path.to_str()),
                        err.to_string()
                    )
                );
                Vec::new()
            }
        };
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
                    Arc::new(RwLock::new(CompartmentCache::new(
                        &dbg,
                        shape.clone(),
                        conf.cache_dir.clone().join(PathBuf::from("compartments")),
                        name.clone(),
                        conf.heel_steps.clone(),
                        conf.trim_steps.clone(),
                        conf.compartment_level_step,
                        conf.model_center_coord.x(),
                        center_max,
                        volume_max,
                        Arc::clone(&thread_pool),
                    ))),
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
                    Arc::new(RwLock::new(DamagedCompartmentCache::new(
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
                        Arc::clone(&thread_pool),
                    ))),
                ))
            })
            .flat_map(|v| v)
            .collect();
        let model_cached = Self {
            dbg: dbg.clone(),
            ship_length_lbp: conf.ship_length_lbp,
            model_center_coord: conf.model_center_coord.clone(),
            draught_min: conf.draught_min,
            hull_draught_step: conf.hull_draught_step,
            bounds_level_step: conf.bounds_level_step,
            cache_dir: conf.cache_dir.clone(),
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
                Arc::clone(&thread_pool),
            ),
            compartments,
            damaged_compartments,
            windage_area,
            displacement_bounded: HashMap::new(),
            compartments_bounded: HashMap::new(),
            thread_pool,
        };
        Ok(model_cached)
    }
    /// reload all shapes
    pub fn reload_shapes(&mut self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "reload_shapes");
        let mut errors = Vec::new();
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let scheduler = self.thread_pool.scheduler();
        // Сначала считаем модели в разных потоках
        for (name, shape) in &self.displacement_shapes {
            let shape = shape.clone();
            let task_results = task_results.clone();
            let handle = scheduler
                .spawn(move || {
                    let mut guard = shape.write();
                    task_results.push(guard.init());
                    Ok(())
                })
                .map_err(|err| {
                    error.pass_with(format!("spawn task displacement_shape {name}"), err)
                });
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => errors.push(err),
            };
        }
        {
            let shape = self.windage_shape.clone();
            let task_results = task_results.clone();
            let handle = scheduler
                .spawn(move || {
                    let mut guard = shape.write();
                    task_results.push(guard.init());
                    Ok(())
                })
                .map_err(|err| error.pass_with(format!("spawn task area_shape"), err.to_string()));
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => errors.push(err),
            };
        }
        for task in tasks {
            log::info!("{}.reload_shapes | join thread {}", &self.dbg, task.name());
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(error);
            }
        }
        if !errors.is_empty() {
            return Err(error.pass_with(
                "rebuild_caches",
                errors
                    .iter()
                    .fold(String::new(), |acc, err| acc + &format!(" error: {err}")),
            ));
        }
        Ok(())
    }
    /// инициализация кэшей заранее посчитанными данными
    pub fn init(&mut self) -> Result<(), Error> {
        //    dbg!(self.dbg.clone(), "init");
        let error = Error::new(self.dbg.clone(), "init");
        self.displacement
            .init()
            .map_err(|err| error.pass_with(format!("displacement.init"), err))?;
        for (name, compartment) in self.compartments.iter_mut() {
            compartment
                .write()
                .init()
                .map_err(|err| error.pass_with(format!("compartment:{name}.init"), err))?
        }
   /*     for (name, damaged_compartment) in self.damaged_compartments.iter_mut() {
            damaged_compartment
                .write()
                .init()
                .map_err(|err| error.pass_with(format!("damaged_compartment:{name}.init"), err))?
        }*/
        self.windage_area
            .init()
            .map_err(|err| error.pass_with(format!("displacement.init"), err))?;
        Ok(())
    }
    /// инициализация кэшей заранее посчитанными данными
    pub fn init_bounded(&mut self, bounds: &Bounds) -> Result<(), Error> {
        //   dbg!(self.dbg.clone(), "init_bounded");
        let error = Error::new(self.dbg.clone(), "init_bounded");
        let bounds_qnt = bounds.len_qnt();
        let displacement_shape = self
            .displacement_shapes
            .get("hull")
            .ok_or(error.err("no displacement_shape"))?;
        let bound_displacement = BoundDisplacementCache::new(
            &self.dbg,
            displacement_shape.clone(),
            self.cache_dir.clone().join("disp_bounded"),
            self.bounds_level_step,
            self.model_center_coord.x(),
            bounds.clone(),
            Arc::clone(&self.thread_pool),
        );
        bound_displacement
            .init()
            .map_err(|err| error.pass_with(format!("bound_displacement.init"), err))?;
        self.displacement_bounded
            .insert(bounds_qnt, Arc::new(RwLock::new(bound_displacement)));
        let mut cache_map = IndexMap::new();
        for (compartment_id, compartment) in &self.compartments {
            let compartment_bounded = compartment
                .read()
                .build_bounded(bounds.clone(), self.bounds_level_step);
            compartment_bounded
                .init()
                .map_err(|err| error.pass_with("compartment_bounded.init", err))?;
            cache_map.insert(
                compartment_id.clone(),
                Arc::new(RwLock::new(compartment_bounded)),
            );
        }
        self.compartments_bounded
            .insert(bounds.len_qnt(), cache_map);
        Ok(())
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
    #[allow(dead_code)]
    pub fn rebuild_caches(&mut self) -> Result<(), Error> {
        log::info!("rebuild_caches begin");
        let error = Error::new(&self.dbg, "rebuild_caches");
        let mut errors = Vec::new();
        // Считаем кэши, они сами по себе многопоточны, поэтому делить на потоки нет смысла
        if let Err(error) = self.displacement.rebuild() {
            errors.push(("displacement".to_owned(), error));
        }
        if let Err(error) = self.windage_area.rebuild() {
            errors.push(("displacement".to_owned(), error));
        }
        for (name, compartment) in &mut self.compartments {
            if let Err(error) = compartment.write().rebuild() {
                errors.push((("compartment ".to_owned() + name), error));
            }
        }
        for (name, compartment) in &mut self.damaged_compartments {
            if let Err(error) = compartment.write().rebuild() {
                errors.push((("damaged_compartment ".to_owned() + name), error));
            }
        }
        /*  for bound_displacement_cache in &mut self.displacement_bounded.values_mut() {
            if let Err(error) = bound_displacement_cache.write().rebuild() {
                errors.push(("displacement_bounded".to_owned(), error));
            }
        }
        for cache_map in &mut self.compartments_bounded.values_mut() {
            for bound_compartment_cache in &mut cache_map.values_mut() {
                if let Err(error) = bound_compartment_cache.write().rebuild() {
                    errors.push(("compartments_bounded".to_owned(), error));
                }
            }
        }*/
        if !errors.is_empty() {
            return Err(error.pass_with(
                "rebuild_caches",
                errors.iter().fold(String::new(), |acc, (key, err)| {
                    format!("{acc}\n\tIn cache {:?} was error: {err}", key)
                }),
            ));
        }
        log::info!("rebuild_caches finish");
        Ok(())
    }
    //
    pub fn body_size(&self) -> Result<(f64, f64, f64), Error> {
        let error = Error::new(&self.dbg, "body_size");
        let (x, y, z, _) = self
            .displacement_shapes
            .get("hull")
            .ok_or(error.err("no displacement_shape"))?
            .read()
            .size()
            .map_err(|err| error.pass_with("loa", err))?;
        Ok((x, y, z))
    }
    //
    #[allow(dead_code)]
    pub fn rebuild_bounds(&mut self, bounds: &Bounds) -> Result<(), Error> {
        let error: Error = Error::new(&self.dbg, "rebuild_bounds");
        let displacement_shape = self
            .displacement_shapes
            .get("hull")
            .ok_or(error.err("no displacement_shape"))?;
        let mut bound_displacement = BoundDisplacementCache::new(
            &self.dbg,
            displacement_shape.clone(),
            self.cache_dir.clone().join("disp_bounded"),
            self.bounds_level_step,
            self.model_center_coord.x(),
            bounds.clone(),
            Arc::clone(&self.thread_pool),
        );
        bound_displacement
            .rebuild()
            .map_err(|err| error.pass_with("bound_displacement.rebuild", err))?;
        self.displacement_bounded
            .insert(bounds.len_qnt(), Arc::new(RwLock::new(bound_displacement)));
        let mut cache_map = IndexMap::new();
        for (compartment_id, compartment) in &self.compartments {
            let mut compartment_bounded = compartment
                .read()
                .build_bounded(bounds.clone(), self.bounds_level_step);
            compartment_bounded
                .rebuild()
                .map_err(|err| error.pass_with("compartment_bounded.rebuild", err))?;
            cache_map.insert(
                compartment_id.clone(),
                Arc::new(RwLock::new(compartment_bounded)),
            );
        }
        self.compartments_bounded
            .insert(bounds.len_qnt(), cache_map);
        Ok(())
    }
    //
    pub fn bounded_windage_area(&self, bounds: &Bounds) -> Result<Vec<f64>, Error> {
        self.windage_area
            .bounded_windage_area(&bounds)
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
    /// Расчет равновесного положения для прочности
    pub fn balance_strength(
        &self,
        query: BalanceStrengthQuery,
    ) -> Result<StrengthBalanceCtx, Error> {
        //   let time = std::time::Instant::now();
        let error = Error::new(&self.dbg, "balance_strength");
        // println!("steps:{_i} time:{:?}", time.elapsed());
        let mut errors = Vec::new();
        let mut liquid = Vec::new();
        let mut bulk = Vec::new();
        let mut gaseous = Vec::new();
        let displacement_bounded = self
            .displacement_bounded
            .get(&query.bounds.len_qnt())
            .ok_or(error.err("no displacement_bounded"))?
            .clone();
        let compartments_bounded = self
            .compartments_bounded
            .get(&query.bounds.len_qnt())
            .ok_or(error.err("no compartments_bounded"))?;
        let scheduler = self.thread_pool.scheduler();
        // расчет эпюр масс для газообразных грузов
        // они не смещаются, поэтому считаем их один раз
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let gaseous_results = Arc::new(Stack::new());
        for cargo in query.gaseous {
            assert!(cargo.mass > 0.);
            let assigment_type = cargo.assigment_type;
            let space_id = cargo.space_id.clone();
            let error_ = error.err(format!("compartment_{space_id} gaseous work"));
            let compartments_bounded = compartments_bounded.clone();
            let results_ = gaseous_results.clone();
            let handle = scheduler
                .spawn(move || {
                    let compartment_bounded = compartments_bounded
                        .get(&space_id)
                        .ok_or(
                            error_.err(format!("compartments_bounded.get no space_id:{space_id}")),
                        )?
                        .read();
                    let volume_bounded = compartment_bounded.get_max().map_err(|err| {
                        error_
                            .pass_with(format!("volume_bounded get_max, space_id:{space_id}"), err)
                    })?;
                    let volume: f64 = volume_bounded.iter().sum();
                    let density = if volume > 0. { cargo.mass / volume } else { 0. };
                    results_.push(strength_balance_eval::gaseous_result::GaseousResult::new(
                        space_id,
                        assigment_type,
                        volume_bounded.into_iter().map(|v| v * density).collect(),
                    ));
                    Ok(())
                })
                .map_err(|err| error.pass_with(format!("scheduler.spawn"), err.to_string()));
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => errors.push(err),
            };
        }
        for task in tasks {
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(error);
            }
        }
        let src_mass_distr = {
            let mut res = query.distr_static.clone();
            while !gaseous_results.is_empty() {
                if let Some(data) = gaseous_results.pop() {
                    res.add_vec(&data.mass_values).map_err(|err| {
                        error.pass_with(format!("mass_distr.add_vec(gaseous)"), err.to_string())
                    })?;
                    gaseous.push(data);
                }
            }
            res
        };
        let mut res_mass_distr = Vec::new();
        let mut res_displacement_distr = Vec::new();
        let (mut trim, mut draught) = (query.trim, query.draught);
        let (mut mass_sum, mut disp_sum) = (0., 0.);
        for _i in 0..50 {
            // trim
            for _j in 0..50 {
                // draught
                let mut tasks: Vec<JoinHandle<_>> = vec![];
                let liquid_results = Arc::new(Stack::new());
                for cargo in &query.liquid {
                    assert!(cargo.mass > 0.);
                    let assigment_type = cargo.assigment_type;
                    let space_id = cargo.space_id.clone();
                    let cargo_type = cargo.cargo_type;
                    let error_ = error.err(format!("compartment_{space_id} liquid work"));
                    let density = cargo.mass / cargo.volume;
                    let compartment = self
                        .compartments
                        .get(&space_id)
                        .ok_or(error.err(format!("no compartment:{space_id}")))?
                        .clone();
                    let compartments_bounded = compartments_bounded.clone();
                    let trim = trim;
                    let volume = cargo.volume;
                    let epsilon = query.epsilon;
                    let results_ = liquid_results.clone();
                    let handle = scheduler
                        .spawn(move || {
                            let compartment_result = compartment
                                .read()
                                .get(0., trim, volume, epsilon)
                                .map_err(|err| error_.pass_with("compartment.get", err))?;
                            let compartment_bounded = compartments_bounded
                                .get(&space_id)
                                .ok_or(error_.err(format!(
                                    "compartments_bounded.get no space_id:{space_id}"
                                )))?
                                .read();
                            let volume_bounded = compartment_bounded
                                .get(compartment_result.level, trim)
                                .map_err(|err| {
                                    error_.pass_with(
                                        format!("compartment_bounded.get, space_id:{space_id}"),
                                        err,
                                    )
                                })?;
                            results_.push(strength_balance_eval::liquid_result::LiquidResult::new(
                                space_id,
                                assigment_type,
                                cargo_type,
                                volume_bounded.into_iter().map(|v| v * density).collect(),
                            ));
                            Ok(())
                        })
                        .map_err(|err| {
                            error.pass_with(format!("scheduler.spawn"), err.to_string())
                        });
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => errors.push(err),
                    };
                }
                let bulk_results = Arc::new(Stack::new());
                for cargo in &query.bulk {
                    assert!(cargo.mass > 0.);
                    let assigment_type = cargo.assigment_type;
                    let space_id = cargo.space_id.clone();
                    let error_ = error.err(format!("compartment_{space_id} bulk work"));
                    let density = cargo.mass / cargo.volume;
                    let compartment = self
                        .compartments
                        .get(&space_id)
                        .ok_or(error.err(format!("no compartment:{space_id}")))?
                        .clone();
                    let compartments_bounded = compartments_bounded.clone();
                    let trim = trim;
                    let volume = cargo.volume;
                    let epsilon = query.epsilon;
                    let results_ = bulk_results.clone();
                    let handle = scheduler
                        .spawn(move || {
                            let compartment_result = compartment
                                .read()
                                .get(0., trim, volume, epsilon)
                                .map_err(|err| error_.pass_with("compartment.get", err))?;
                            let compartment_bounded = compartments_bounded
                                .get(&space_id)
                                .ok_or(error_.err(format!(
                                    "compartments_bounded.get no space_id:{space_id}"
                                )))?
                                .read();
                            let volume_bounded = compartment_bounded
                                .get(compartment_result.level, trim)
                                .map_err(|err| {
                                    error_.pass_with(
                                        format!("compartment_bounded.get, space_id:{space_id}"),
                                        err,
                                    )
                                })?;
                            results_.push(strength_balance_eval::bulk_result::BulkResult::new(
                                space_id,
                                assigment_type,
                                volume_bounded.into_iter().map(|v| v * density).collect(),
                            ));
                            Ok(())
                        })
                        .map_err(|err| {
                            error.pass_with(format!("scheduler.spawn"), err.to_string())
                        });
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => errors.push(err),
                    };
                }
                let hull_results = Arc::new(Stack::new());
                let results_ = hull_results.clone();
                let displacement_bounded = displacement_bounded.clone();
                let handle = scheduler
                    .spawn(move || {
                        results_.push(displacement_bounded.read().get(draught, trim));
                        Ok(())
                    })
                    .map_err(|err| {
                        error.pass_with(
                            format!("scheduler.spawn displacement_bounded"),
                            err.to_string(),
                        )
                    });
                match handle {
                    Ok(task) => tasks.push(task),
                    Err(err) => errors.push(err),
                };
                for task in tasks {
                    if let Err(err) = task.join() {
                        let error = error.pass_with("task join", err.to_string());
                        log::error!("{}", error);
                        errors.push(error);
                    }
                }
                res_mass_distr = src_mass_distr.clone();
                while !liquid_results.is_empty() {
                    if let Some(data) = liquid_results.pop() {
                        res_mass_distr.add_vec(&data.mass_values).map_err(|err| {
                            error.pass_with("mass_distr.add_vec(liquid)", err.to_string())
                        })?;
                        liquid.push(data);
                    }
                }
                while !bulk_results.is_empty() {
                    if let Some(data) = bulk_results.pop() {
                        res_mass_distr.add_vec(&data.mass_values).map_err(|err| {
                            error.pass_with("mass_distr.add_vec(bulk)", err.to_string())
                        })?;
                        bulk.push(data);
                    }
                }
                res_displacement_distr = hull_results
                    .pop()
                    .ok_or(error.err("no hull_result"))?
                    .map_err(|err| error.pass_with("hull_result", err))?;
                if !errors.is_empty() {
                    return Err(error.pass_with(
                        "balance_strength",
                        errors
                            .iter()
                            .fold(String::new(), |acc, err| acc + &format!(" error: {err}")),
                    ));
                }
                if res_mass_distr.len() != res_displacement_distr.len()
                    || res_mass_distr.len() != query.bounds.len_qnt()
                {
                    let error = error.err("res_mass_distr.len()");
                    log::error!("{error}");
                    return Err(error);
                }
                mass_sum = res_mass_distr.iter().sum();
                disp_sum = res_displacement_distr.iter().sum::<f64>() * query.water_density;
                if mass_sum <= 0. || disp_sum <= 0. {
                    return Err(error.err("mass_sum <= 0 || disp_sum <= 0"));
                };
                let delta_w: f64 = (mass_sum - disp_sum) / mass_sum;
                if delta_w.abs() <= query.epsilon {
                    println!("bfgsdb draught: {_j}, {draught}, {delta_w}, {mass_sum}, {disp_sum}");
                    break;
                }
                draught = 0.5_f64.max(draught + draught * delta_w);
            }
            let (mut mass_moment, mut disp_moment) = (0., 0.);
            for (i, bound) in query.bounds.iter().enumerate() {
                let center_x = bound.center().ok_or(error.err("bound.center"))?;
                mass_moment += center_x * res_mass_distr[i];
                disp_moment += center_x * res_displacement_distr[i];
            }
            let (mass_x, disp_x) = (
                mass_moment / mass_sum,
                disp_moment * query.water_density / disp_sum,
            );
            let delta_x = mass_x - disp_x;
            if delta_x.abs() <= query.epsilon {
                println!("bfgsdb trim: {_i}, {trim}, {delta_x}, {mass_x}, {disp_x}");
                break;
            }
            trim += delta_x / 10.;
        }

        /*  let mut total_force = mass_distr;
                displacement_distr.mul_single(query.water_density);
                /*     let volume_sum: f64 = volume_values.iter().sum();
                    let mass_sum: f64 = mass_values.iter().sum();
                    let multipler = if volume_sum > 0. { mass_sum/volume_sum } else { 1. };
                    dbg!(volume_sum, mass_sum, multipler);
                    volume_values.mul_single(multipler);
                */
                total_force.sub_vec(&displacement_distr)?;
                total_force.mul_single(9.81); // gravity
                let shear_force = total_force.sum_above();
                log::trace!("\t ShearForce result:{:?}", shear_force);
                println!("\n\n ShearForce result\n");
                shear_force.iter().for_each(|b| print!("{:.3} ", b));
                let mut bending_moment: Vec<f64> = shear_force.integral_sum();
                bending_moment = bending_moment
                    .into_iter()
                    .zip(query.bounds.iter())
                    .map(|(v, b)| v * b.length().unwrap_or(0.) / 2.)
                    .collect();
                println!("\n\n BendingMoment result\n");
                bending_moment.iter().for_each(|b| print!("{:.3} ", b));
        */
        Ok(StrengthBalanceCtx {
            displacement_distr: res_displacement_distr,
            bulk,
            liquid,
            gaseous,
        })
    }
    /// Расчет равновесного положения для остойчивости
    pub fn balance_stability(
        &self,
        query: BalanceStabilityQuery,
    ) -> Result<BalanceStabilityResult, Error> {
        //   let time = std::time::Instant::now();
        let error = Error::new(&self.dbg, "balance_stability");
        let FloatingPositionResult {
            heel,
            trim,
            draught_mid,
            precision,
            displacement,
            displacement_center,
            area_wl,
            area_wl_center,
            length_wl,
            breadth_wl,
            rad_long,
            rad_trans,
        } = self
            .floating_position(FloatingPositionQuery {
                water_density: query.water_density,
                mass_const: query.mass_const,
                moment_const: query.moment_const,
                bulk: query.bulk.clone(),
                liquid: query.liquid.clone(),
                grain_bulkhead: query.grain_bulkhead.clone(), // TODO
                damaged_compartment: Vec::new(),
                epsilon: query.epsilon,
            })
            .map_err(|err| error.pass_with("self.floating_position", err))?;
        // println!("steps:{_i} time:{:?}", time.elapsed());
        let (draught_bow, draught_stern, draught_mean) = Draught::new(
            self.model_center_coord.x(),
            self.ship_length_lbp,
            draught_mid,
            area_wl_center.x(),
            area_wl_center.y(),
            heel,
            trim,
        )
        .calculate();
        let trim_degree = trim;
        let trim_meter = trim.to_radians().tan() * self.ship_length_lbp;
        let mut errors = Vec::new();
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let liquid_results = Arc::new(Stack::new());
        let bulk_results = Arc::new(Stack::new());
        let scheduler = self.thread_pool.scheduler();
        for cargo in query.liquid {
            assert!(cargo.mass > 0.);
            let assigned_id = cargo.assigned_id;
            let space_id = cargo.space_id.clone();
            let error_ = error.err(format!("compartment_{space_id} liquid work"));
            let compartment = self
                .compartments
                .get(&space_id)
                .ok_or(error.err(format!("no compartment:{space_id}")))?
                .clone();
            let trim = trim;
            let volume = cargo.volume;
            let epsilon = query.epsilon;
            let results_ = liquid_results.clone();
            let handle = scheduler
                .spawn(move || {
                    let compartment_result = compartment
                        .read()
                        .get(0., trim, volume, epsilon)
                        .map_err(|err| error_.pass_with("compartment.get", err))?;
                    results_.push(stability_result::LiquidResult::new(
                        //     cargo_id,
                        assigned_id,
                        //     compartment_result.volume_center,
                        compartment_result.inertia_long_y,
                        compartment_result.inertia_trans_x,
                    ));
                    Ok(())
                })
                .map_err(|err| error.pass_with(format!("scheduler.spawn"), err.to_string()));
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => errors.push(err),
            };
        }
        for cargo in query.bulk {
            assert!(cargo.mass > 0.);
            let assigned_id = cargo.assigned_id;
            let space_id = cargo.space_id.clone();
            let error_ = error.err(format!("compartment_{space_id} bulk work"));
            let compartment = self
                .compartments
                .get(&space_id)
                .ok_or(error.err(format!("no compartment:{space_id}")))?
                .clone();
            let trim = trim;
            let volume = cargo.volume;
            let epsilon = query.epsilon;
            let results_ = bulk_results.clone();
            let handle = scheduler
                .spawn(move || {
                    let compartment_result = compartment
                        .read()
                        .get(0., trim, volume, epsilon)
                        .map_err(|err| error_.pass_with("compartment.get", err))?;
                    results_.push(stability_result::BulkResult::new(
                        //       cargo_id,
                        space_id,
                        assigned_id,
                        compartment_result.level,
                        //       compartment_result.volume_center,
                    ));
                    Ok(())
                })
                .map_err(|err| error.pass_with(format!("scheduler.spawn"), err.to_string()));
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => errors.push(err),
            };
        }
        for task in tasks {
            log::info!("{}.balance | join thread {}", &self.dbg, task.name());
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(error);
            }
        }
        if !errors.is_empty() {
            return Err(error.pass_with(
                "rebuild_caches",
                errors
                    .iter()
                    .fold(String::new(), |acc, err| acc + &format!(" error: {err}")),
            ));
        }
        let liquid = {
            let mut result = Vec::new();
            while !liquid_results.is_empty() {
                if let Some(data) = liquid_results.pop() {
                    result.push(data);
                }
            }
            result
        };
        let bulk = {
            let mut result = Vec::new();
            while !bulk_results.is_empty() {
                if let Some(data) = bulk_results.pop() {
                    result.push(data);
                }
            }
            result
        };
        Ok(BalanceStabilityResult {
            roll: heel,
            trim_degree,
            trim_meter,
            draught_mid,
            draught_bow,
            draught_stern,
            draught_mean,
            displacement,
            displacement_center,
            bulk,
            liquid,
            area_wl,
            area_wl_center,
            length_wl,
            breadth_wl,
            rad_long,
            rad_trans,
        })
    }
    /*  /// Расчет равновесного положения
    pub(crate) fn floating_position(
        &self,
        query: FloatingPositionQuery,
    ) -> Result<FloatingPositionResult, Error> {
        let error = Error::new(&self.dbg, "floating_position");
        if query.water_density <= 0. {
            return Err(error.err("water_density <= 0."));
        }
        // постоянная масса
        let mass_const = query.mass_const;
        // постоянный момент
        let moment_const = query.moment_const;
        // Считаем сыпучие грузы.
        // На них крен и дифферент не влияет.
        let moment_bulk = self
            .moment_bulk(&query.bulk, query.epsilon)
            .map_err(|err| error.pass_with("self.bulk_moment", err))?;
        let mass_bulk = query.bulk.iter().map(|v| v.mass).sum::<f64>();
        let mass_liquid = query.liquid.iter().map(|v| v.mass).sum::<f64>();
        let mut heel = 0.0;
        let mut trim = 0.0;
        let mut draught = self.draught_min;
        let mut step_trim = 0.5;
        let mut step_heel = 1.0;
        let mut d_v: Option<f64> = None;
        let mut d_m: Option<f64> = None;
        for _i in 1..=1000 {
            let epsilon = (step_trim + step_heel) / 10.;
            // учет смещения жидкости
            let moment_liquid = self
                .moment_liquid(&query.liquid, heel, trim, epsilon)
                .map_err(|err| error.pass_with("self.moment_liquid", err))?;
            // учет изменения водоизмещения из-за поврежденных отсеков
            // поврежденные отсеки есть только в аварийном расчете, иначе список пустой
            let (mass_damaged_compartment, moment_damaged_compartment) = self
                .calc_damaged_compartments(
                    &query.damaged_compartment,
                    heel,
                    trim,
                    draught,
                    query.water_density,
                )
                .map_err(|err| error.pass_with("self.calc_damaged_compartments", err))?;
            let mass_sum = mass_const + mass_bulk + mass_liquid + mass_damaged_compartment;
            let displacement = mass_sum / query.water_density;
            // считаем корпус с учетом изменения массы
            let disp_result = self
                .displacement
                .get(heel, trim, mass_sum / query.water_density, epsilon)
                .map_err(|err| {
                    error.pass_with(
                        format!("self.displacement.get heel:{heel} trim:{trim} displacement:{displacement}"),
                        err,
                    )
                })?;
            let (new_draught, cb) = (disp_result.draught, disp_result.volume_center);
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
                moment_sum.to_pos(mass_sum)
            };
            // Определение невязки
            let cg_h = {
                // Через центр плавучести CB проводится горизонтальная плоскость
                let my_plane = HalfSpace::new(Vector3::z_axis());
                let cg_local = cg - cb;
                let cg_local = rotation.transform_point(&cg_local.into());
                let cg_h_local = my_plane.project_local_point(&cg_local.into(), false).point;
                let cg_h_local = rotation.inverse_transform_point(&cg_h_local.into());
                let cg_h = cb + cg_h_local.into();
                let precision = (cg_h - cb).len();
                if precision < query.epsilon && query.epsilon <= epsilon {
                    let result = FloatingPositionResult {
                        heel,
                        trim,
                        draught_mid: new_draught,
                        precision,
                        displacement,
                        displacement_center: disp_result.volume_center,
                        area_wl: disp_result.area_wl,
                        area_wl_center: disp_result.area_wl_center,
                        length_wl: disp_result.length_wl,
                        breadth_wl: disp_result.breadth_wl,
                        rad_long: disp_result.inertia_long_y / displacement,
                        rad_trans: disp_result.inertia_trans_x / displacement,
                    };
                    return Ok(result);
                }
                cg_h
            };
            // Определение посадки судна для следующего шага
            let cb_v = {
                // Через центр плавучести CG проводится вертикальная плоскость параллельная основной линии
                let my_plane = HalfSpace::new(Vector3::y_axis());
                let cb_local = cb - cg;
                let cb_local = rotation.transform_point(&cb_local.into());
                let cg_v_local = my_plane.project_local_point(&cb_local.into(), false).point;
                let cg_v = rotation.inverse_transform_point(&cg_v_local.into());
                let cb_v = cg + cg_v.into();
                cb_v
            };
            let cb_m = {
                // Через центр плавучести CG проводится вертикальная плоскость параллельная миделю
                let my_plane = HalfSpace::new(Vector3::x_axis());
                let cb_local = cb - cg;
                let cb_local = rotation.transform_point(&cb_local.into());
                let cb_m_local = my_plane.project_local_point(&cb_local.into(), false).point;
                let cb_m = rotation.inverse_transform_point(&cb_m_local.into());
                let cb_m = cg + cb_m.into();
                cb_m
            };
            // проекция точки cg_m на вертикальную плоскость параллельную основной линии
            let cg_m_h = {
                // Через центр плавучести CG проводится вертикальная плоскость параллельная основной линии
                let my_plane = HalfSpace::new(Vector3::y_axis());
                let cg_m_local = cb_m - cg;
                let cg_m_local = rotation.transform_point(&cg_m_local.into());
                let cg_m_h_local = my_plane
                    .project_local_point(&cg_m_local.into(), false)
                    .point;
                let cg_m_h = rotation.inverse_transform_point(&cg_m_h_local.into());
                let cg_m_h = cg + cg_m_h.into();
                cg_m_h
            };
            let new_d_v = cg_h.x() - cb_v.x();
            let new_d_m = cg_m_h.y() - cb_m.y();
            if let Some(old_d_v) = d_v {
                if old_d_v.signum() != new_d_v.signum() {
                    step_trim *= 0.5;
                }
            }
            d_v = Some(new_d_v);
            if let Some(old_d_m) = d_m {
                if old_d_m.signum() != new_d_m.signum() {
                    step_heel *= 0.5;
                }
            }
            d_m = Some(new_d_m);
            trim = trim + step_trim * new_d_v.signum();
            heel = heel + step_heel * new_d_m.signum();
            draught = new_draught;
        }
        Err(error.err(format!("query:{:?} error: no result", query)))
    }
    */
    /// Расчет равновесного положения
    pub(crate) fn floating_position(
        &self,
        query: FloatingPositionQuery,
    ) -> Result<FloatingPositionResult, Error> {
        let error = Error::new(&self.dbg, "floating_position");
        if query.water_density <= 0. {
            return Err(error.err("water_density <= 0."));
        }
        // Считаем сыпучие грузы.
        // На них крен и дифферент не влияет.
        let moment_bulk = self
            .moment_bulk(&query.bulk, query.epsilon)
            .map_err(|err| error.pass_with("self.bulk_moment", err))?;
        let mass_bulk = query.bulk.iter().map(|v| v.mass).sum::<f64>();
        let mass_liquid = query.liquid.iter().map(|v| v.mass).sum::<f64>();
        let mut heel = 0.0;
        let mut trim = 0.0;
        let mut draught = self.draught_min;
        let mut step_trim = 0.5;
        let mut step_heel = 1.0;
        let mut d_v: Option<f64> = None;
        let mut d_m: Option<f64> = None;
        for _i in 1..=1000 {
            let epsilon = (step_trim + step_heel) / 10.;
            let (new_draught, new_d_v, new_d_m, _, displacement, disp_result) = self
                .position(
                    heel,
                    trim,
                    draught,
                    query.water_density,
                    epsilon,
                    query.mass_const + mass_bulk + mass_liquid, // постоянная масса
                    query.moment_const + moment_bulk,           // постоянный момент
                    &query.liquid,
                    &query.damaged_compartment,
                )
                .map_err(|err| error.pass(err))?;
            if query.epsilon <= epsilon {
                let precision = (new_d_v.powi(2) + new_d_m.powi(2)).sqrt();
                if precision < query.epsilon {
                    let result = FloatingPositionResult {
                        heel,
                        trim,
                        draught_mid: new_draught,
                        precision,
                        displacement,
                        displacement_center: disp_result.volume_center,
                        area_wl: disp_result.area_wl,
                        area_wl_center: disp_result.area_wl_center,
                        length_wl: disp_result.length_wl,
                        breadth_wl: disp_result.breadth_wl,
                        rad_long: disp_result.inertia_long_y / displacement,
                        rad_trans: disp_result.inertia_trans_x / displacement,
                    };
                    return Ok(result);
                }
            }
            if let Some(old_d_v) = d_v {
                if old_d_v.signum() != new_d_v.signum() {
                    step_trim *= 0.5;
                }
            }
            d_v = Some(new_d_v);
            if let Some(old_d_m) = d_m {
                if old_d_m.signum() != new_d_m.signum() {
                    step_heel *= 0.5;
                }
            }
            d_m = Some(new_d_m);
            trim = trim + step_trim * new_d_v.signum();
            heel = heel + step_heel * new_d_m.signum();
            draught = new_draught;
        }
        Err(error.err(format!("query:{:?} error: no result", query)))
    }
    /// Расчет диаграммы статической остойчивости
    pub(crate) fn dso(
        &self,
        query: FloatingPositionQuery,
        draught: f64,
        trim: f64,
        heel_max: f64,
    ) -> Result<Vec<(f64, f64)>, Error> {
        let error = Error::new(&self.dbg, "floating_position");
        if query.water_density <= 0. {
            return Err(error.err("water_density <= 0."));
        }
        // Считаем сыпучие грузы.
        // На них крен и дифферент не влияет.
        let moment_bulk = self
            .moment_bulk(&query.bulk, query.epsilon)
            .map_err(|err| error.pass_with("self.bulk_moment", err))?;
        let mass_bulk = query.bulk.iter().map(|v| v.mass).sum::<f64>();
        let mass_liquid = query.liquid.iter().map(|v| v.mass).sum::<f64>();
        let max = (heel_max * 10.) as i32;
        let min = -max;
        let heel = (min..=max).map(|i| i as f64 * 0.1).collect::<Vec<f64>>();
        let mut dso = Vec::new();
        for heel in heel {
            let mut trim = trim;
            let mut draught = draught;
            let mut step_trim = 0.5;
            let mut d_v: Option<f64> = None;
            for _i in 1..=100 {
                let epsilon = step_trim / 10.;
                let (new_draught, new_d_v, _, cg, _, disp_result) = self
                    .position(
                        heel,
                        trim,
                        draught,
                        query.water_density,
                        epsilon,
                        query.mass_const + mass_bulk + mass_liquid, // постоянная масса
                        query.moment_const + moment_bulk,           // постоянный момент
                        &query.liquid,
                        &query.damaged_compartment,
                    )
                    .map_err(|err| error.pass(err))?;
                if query.epsilon <= epsilon {
                    let precision = new_d_v.abs();
                    if precision < query.epsilon {
                        let [_, yg, zg] = cg.values();
                        let [_, yc, zc] = disp_result.volume_center.values();
                        let l = if heel.abs() > f64::EPSILON {
                            let ctg_phy = 1.0 / heel.to_radians().tan();
                            (yg * ctg_phy + zg - yc * ctg_phy - zc) / (1. + ctg_phy.powi(2)).sqrt()
                        } else {
                            yg - yc
                        };
                        dso.push((heel, l));
                        break;
                    }
                }
                if let Some(old_d_v) = d_v {
                    if old_d_v.signum() != new_d_v.signum() {
                        step_trim *= 0.5;
                    }
                }
                d_v = Some(new_d_v);
                trim = trim + step_trim * new_d_v.signum();
                draught = new_draught;
            }
        }
        println!("\nmodel_cached dso: ");
        for &(angle, value) in dso.iter() {
            println!("{angle} {value}");
        }
        Ok(dso)
    }
    /// Расчет итерации в расчете равновесного положения и диаграммы
    fn position(
        &self,
        heel: f64,
        trim: f64,
        draught: f64,
        water_density: f64,
        epsilon: f64,
        mass_sum: f64,        // постоянная масса mass_const + mass_bulk + mass_liquid
        moment_sum: Position, // постоянный момент moment_const + moment_bulk
        liquid: &Vec<LiquidData>,
        damaged_compartment: &Vec<String>,
    ) -> Result<(f64, f64, f64, Position, f64, DisplacementCacheResult), Error> {
        //(draught, d_v, d_m, cg, displacement, disp_result)
        let error = Error::new(&self.dbg, "_floating_position");
        // учет смещения жидкости
        let moment_liquid = self
            .moment_liquid(&liquid, heel, trim, epsilon)
            .map_err(|err| error.pass_with("self.moment_liquid", err))?;
        // учет изменения водоизмещения из-за поврежденных отсеков
        // поврежденные отсеки есть только в аварийном расчете, иначе список пустой
        let (mass_damaged_compartment, moment_damaged_compartment) = self
            .calc_damaged_compartments(&damaged_compartment, heel, trim, draught, water_density)
            .map_err(|err| error.pass_with("self.calc_damaged_compartments", err))?;
        let mass_sum = mass_sum + mass_damaged_compartment;
        let displacement = mass_sum / water_density;
        // считаем корпус с учетом изменения массы
        let disp_result = self
            .displacement
            .get(heel, trim, mass_sum / water_density, epsilon)
            .map_err(|err| {
                error.pass_with(
                    format!(
                        "self.displacement.get heel:{heel} trim:{trim} displacement:{displacement}"
                    ),
                    err,
                )
            })?;
        let (draught, cb) = (disp_result.draught, disp_result.volume_center);
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
            let moment_sum = moment_sum + moment_liquid + moment_damaged_compartment;
            moment_sum.to_pos(mass_sum)
        };
        // Определение невязки
        let cg_h = {
            // Через центр плавучести CB проводится горизонтальная плоскость
            let my_plane = HalfSpace::new(Vector3::z_axis());
            let cg_local = cg - cb;
            let cg_local = rotation.transform_point(&cg_local.into());
            let cg_h_local = my_plane.project_local_point(&cg_local.into(), false).point;
            let cg_h_local = rotation.inverse_transform_point(&cg_h_local.into());
            let cg_h = cb + cg_h_local.into();
            cg_h
        };
        // Определение посадки судна для следующего шага
        let cb_v = {
            // Через центр плавучести CG проводится вертикальная плоскость параллельная основной линии
            let my_plane = HalfSpace::new(Vector3::y_axis());
            let cb_local = cb - cg;
            let cb_local = rotation.transform_point(&cb_local.into());
            let cg_v_local = my_plane.project_local_point(&cb_local.into(), false).point;
            let cg_v = rotation.inverse_transform_point(&cg_v_local.into());
            let cb_v = cg + cg_v.into();
            cb_v
        };
        let cb_m = {
            // Через центр плавучести CG проводится вертикальная плоскость параллельная миделю
            let my_plane = HalfSpace::new(Vector3::x_axis());
            let cb_local = cb - cg;
            let cb_local = rotation.transform_point(&cb_local.into());
            let cb_m_local = my_plane.project_local_point(&cb_local.into(), false).point;
            let cb_m = rotation.inverse_transform_point(&cb_m_local.into());
            let cb_m = cg + cb_m.into();
            cb_m
        };
        // проекция точки cg_m на вертикальную плоскость параллельную основной линии
        let cg_m_h = {
            // Через центр плавучести CG проводится вертикальная плоскость параллельная основной линии
            let my_plane = HalfSpace::new(Vector3::y_axis());
            let cg_m_local = cb_m - cg;
            let cg_m_local = rotation.transform_point(&cg_m_local.into());
            let cg_m_h_local = my_plane
                .project_local_point(&cg_m_local.into(), false)
                .point;
            let cg_m_h = rotation.inverse_transform_point(&cg_m_h_local.into());
            let cg_m_h = cg + cg_m_h.into();
            cg_m_h
        };
        let d_v = cg_h.x() - cb_v.x();
        let d_m = cg_m_h.y() - cb_m.y();
        Ok((draught, d_v, d_m, cg, displacement, disp_result))
    }
    // Считаем сыпучие грузы.
    // На них крен и дифферент не влияет.
    fn moment_bulk(&self, bulks: &Vec<BulkData>, epsilon: f64) -> Result<Moment, Error> {
        let error = Error::new(&self.dbg, "moment_bulk");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let mut values = Vec::new();
        let scheduler = self.thread_pool.scheduler();
        for bulk in bulks {
            match self.compartments.get(&bulk.space_id) {
                Some(compartment) => {
                    let task_results = task_results.clone();
                    let epsilon = epsilon.clone();
                    let space_id = bulk.space_id.clone();
                    let mass = bulk.mass;
                    let volume = bulk.volume;
                    let compartment = compartment.clone();
                    let handle = scheduler
                        .spawn(move || {
                            task_results.push((
                                space_id,
                                mass,
                                compartment.read().get(0., 0., volume, epsilon),
                            ));
                            Ok(())
                        })
                        .map_err(|err| {
                            error.pass_with(format!("spawn for {}", bulk.space_id), err)
                        });
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => {
                            let error =
                                error.pass_with(format!("handle for {}", bulk.space_id), err);
                            log::error!("{}", error);
                            errors.push(error);
                        }
                    };
                }
                None => {
                    let error = error.err(format!("no compartment: {}", bulk.space_id));
                    log::error!("{}", error);
                    errors.push(error);
                }
            }
        }
        for task in tasks {
            log::info!("{}.moment_bulk | join thread {}", &self.dbg, task.name());
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(error);
            }
        }
        while !task_results.is_empty() {
            if let Some((space_id, mass, data)) = task_results.pop() {
                let CompartmentCacheResult {
                    level,
                    volume_center,
                    ..
                } = match data {
                    Ok(data) => data,
                    Err(err) => {
                        let error = error
                            .pass_with(format!("task_results data in {space_id}"), err.to_string());
                        log::error!("{}", error);
                        errors.push(error);
                        continue;
                    }
                };
                values.push((space_id, level, Moment::from_pos(volume_center, mass)));
            }
        }
        if !errors.is_empty() {
            return Err(error.pass_with(
                "moment_bulk",
                errors
                    .iter()
                    .fold(String::new(), |acc, err| acc + &format!("\n{}", err)),
            ));
        }
        let sum_moment: Moment = values.iter().map(|(_, _, m)| *m).sum();
        Ok(sum_moment)
    }
    /// Считаем жидкие грузы
    fn moment_liquid(
        &self,
        liquids: &Vec<LiquidData>,
        heel: f64,
        trim: f64,
        epsilon: f64,
    ) -> Result<Moment, Error> {
        let error = Error::new(&self.dbg, "moment_liquid");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let mut values = Vec::new();
        let scheduler = self.thread_pool.scheduler();
        for liquid in liquids {
            match self.compartments.get(&liquid.space_id) {
                Some(compartment) => {
                    let task_results = task_results.clone();
                    let epsilon = epsilon.clone();
                    let space_id = liquid.space_id.clone();
                    let mass = liquid.mass;
                    let volume = liquid.volume;
                    let compartment = compartment.clone();
                    let handle = scheduler
                        .spawn(move || {
                            task_results.push((
                                space_id,
                                mass,
                                compartment.read().get(heel, trim, volume, epsilon),
                            ));
                            Ok(())
                        })
                        .map_err(|err| {
                            error.pass_with(format!("spawn for {}", liquid.space_id), err)
                        });
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => {
                            let error =
                                error.pass_with(format!("handle for {}", liquid.space_id), err);
                            log::error!("{}", error);
                            errors.push(error);
                        }
                    };
                }
                None => {
                    let error = error.err(format!("no compartment: {}", liquid.space_id));
                    log::error!("{}", error);
                    errors.push(error);
                }
            }
        }
        for task in tasks {
            log::info!("{}.moment_liquid | join thread {}", &self.dbg, task.name());
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(error);
            }
        }
        while !task_results.is_empty() {
            if let Some((space_id, mass, data)) = task_results.pop() {
                let CompartmentCacheResult {
                    level,
                    volume_center,
                    ..
                } = match data {
                    Ok(data) => data,
                    Err(err) => {
                        let error = error
                            .pass_with(format!("task_results data in {space_id}"), err.to_string());
                        log::error!("{}", error);
                        errors.push(error);
                        continue;
                    }
                };
                values.push((space_id, level, Moment::from_pos(volume_center, mass)));
            }
        }
        if !errors.is_empty() {
            return Err(error.pass_with(
                "moment_liquid",
                errors
                    .iter()
                    .fold("errors:".to_string(), |acc, err| acc + &format!("\n{}", err)),
            ));
        }
        let sum_moment: Moment = values.iter().map(|(_, _, m)| *m).sum();
        Ok(sum_moment)
    }
    /// Считаем поврежденные отсеки
    fn calc_damaged_compartments(
        &self,
        damaged_compartments: &Vec<String>,
        heel: f64,
        trim: f64,
        draught: f64,
        water_density: f64,
    ) -> Result<(f64, Moment), Error> {
        let error = Error::new(&self.dbg, "moment_damaged_compartments");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let mut values = Vec::new();
        let scheduler = self.thread_pool.scheduler();
        for damaged_compartment in damaged_compartments {
            match self.damaged_compartments.get(damaged_compartment) {
                Some(compartment) => {
                    let task_results = task_results.clone();
                    let space_id = damaged_compartment.clone();
                    let compartment = compartment.clone();
                    let _error = error.clone();
                    let handle = scheduler
                        .spawn(move || {
                            task_results
                                .push((space_id, compartment.read().get(heel, trim, draught)));
                            Ok(())
                        })
                        .map_err(|err| {
                            _error.pass_with(
                                format!("spawn for {}", damaged_compartment.clone()),
                                err,
                            )
                        });
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => {
                            let error =
                                error.pass_with(format!("handle for {damaged_compartment}"), err);
                            log::error!("{}", error);
                            errors.push(error);
                        }
                    };
                }
                None => {
                    let error = error.err(format!("no compartment: {damaged_compartment}"));
                    log::error!("{}", error);
                    errors.push(error);
                }
            }
        }
        for task in tasks {
            log::info!(
                "{}.calc_damaged_compartments | join thread {}",
                &self.dbg,
                task.name()
            );
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(error);
            }
        }
        while !task_results.is_empty() {
            if let Some((space_id, data)) = task_results.pop() {
                let (mass, position) = match data {
                    Ok((volume, position)) => (volume * water_density, position),
                    Err(err) => {
                        let error = error
                            .pass_with(format!("task_results data in {space_id}"), err.to_string());
                        log::error!("{}", error);
                        errors.push(error);
                        continue;
                    }
                };
                values.push((space_id, mass, Moment::from_pos(position, mass)));
            }
        }
        if !errors.is_empty() {
            return Err(error.pass_with(
                "calc_damaged_compartments",
                errors
                    .iter()
                    .fold(String::new(), |acc, err| acc + &format!("\n{}", err)),
            ));
        }
        let result = values.iter().fold(
            (0., Moment::zero()),
            |(mass_sum, moment_sum), (_, mass, moment)| (mass_sum + mass, moment_sum + *moment),
        );
        Ok(result)
    }
}
