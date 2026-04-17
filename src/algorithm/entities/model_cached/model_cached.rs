use super::{LocalCache, ModelCachedConf};
use crate::{
    algorithm::{
        entities::{
            AddVec, Bounds, Moment, Position,
            model_cached::{
                AreaResult, AreaShape, CompartmentBoundCache, CompartmentCache,
                DamagedCompartmentCache, DisplacementBoundCache, DisplacementCache,
                DisplacementCacheResult, DisplacementShape, Draught, HoldCompartmentBoundCache,
                HoldCompartmentCache, Shape, WindageArea,
            },
            ship_model::{
                stability_result::{BalanceStabilityResult, BulkResult, LiquidResult},
                *,
            },
        },
        eval::strength::{StrengthBalanceCtx, balance},
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
    /// Массовое водоизмещение, т
    pub mass: f64,
    /// Смещение центра массы, м
    pub mass_center: Position,
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
/// Результат расчета DSO
#[derive(Debug)]
pub(crate) struct DsoResult {
    /// Крен с учетом DSO
    pub heel: f64,
    /// DSO
    pub dso: Vec<(f64, f64)>,
    /// Угол входа в воду палубы    
    pub entry_angle: Vec<(f64, f64)>,
    /// Угол входа в воду открытых отверстий             
    pub flooding_angle: Vec<(f64, f64)>,
}
///
/// See [sal_3dlib::props::Attributes] to get more details about what the attribute type is.
pub struct ModelCached {
    dbg: Dbg,
    /// Ship length between perpendiculars
    ship_length_lbp: f64,
    /// 3d model initial position in 3D space (midel).
    model_x: f64,
    /// Waterline coord Z in 3D space (midel) initial position.
    draught_min: f64,
    /// Draught step for hull
    hull_draught_step: f64,
    /// Level step for bounds
    bounds_level_step: f64,
    /// Directory containing [super::ModelCached] caches.
    cache_dir: PathBuf,
    /// Angles for DSO
    dso_angles: Vec<f64>,
    /// Privides access to structure of the 3D element
    displacement_shapes: IndexMap<String, Arc<RwLock<DisplacementShape>>>,
    windage_shape: Arc<RwLock<AreaShape>>,
    /// Provides a number of calculations:
    /// - cache for model, [heel, trim, draught, volume, x, y, z, area, x, y, z, l_x, l_y ]
    displacement: DisplacementCache,
    /// - cache for compartments, [index of compartments, [heel, trim, level, volume, x, y, z, i_x, i_y ]]
    compartments: IndexMap<String, Arc<RwLock<CompartmentCache>>>,
    /// Композитные отсеки трюмов
    hold_compartments: IndexMap<String, Arc<RwLock<HoldCompartmentCache>>>,
    /// - cache for damaged compartments, [index of compartments, [heel, trim, draught, volume, x, y, z ]]
    damaged_compartments: IndexMap<String, Arc<RwLock<DamagedCompartmentCache>>>,
    /// - cache for windage area
    windage_area: WindageArea,
    /// - cache for bounds of model, [qnt_bounds, cache]
    displacement_bounded: IndexMap<usize, Arc<RwLock<DisplacementBoundCache>>>,
    /// - cache for bounds of compartments, [qnt_bounds, [code, cache]]
    compartments_bounded: IndexMap<usize, IndexMap<String, Arc<RwLock<CompartmentBoundCache>>>>,
    /// Композитные отсеки трюмов, разбиение по шпациям
    hold_compartments_bounded:
        IndexMap<usize, IndexMap<String, Arc<RwLock<HoldCompartmentBoundCache>>>>,
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
        let model_x = Some(conf.model_x);
        let displacement_shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
            &dbg,
            conf.model_dir.clone().join(PathBuf::from("hull.stl")),
            model_x,
            conf.model_scale,
        )));
        displacement_shapes.insert("hull".to_owned(), displacement_shape.clone());
        let windage_shape = Arc::new(RwLock::new(AreaShape::new_uninit(
            &dbg,
            conf.model_dir.clone().join(PathBuf::from("hull.stl")),
            Some(conf.model_dir.clone().join(PathBuf::from("additionals"))),
            model_x,
            conf.model_scale,
        )));
        let windage_area = WindageArea::new(
            &dbg,
            windage_shape.clone(),
            conf.cache_dir.clone(),
            conf.draught_min,
            Arc::clone(&thread_pool),
        );
        let path = conf.model_dir.clone().join(PathBuf::from("compartments"));
        let pathes: Vec<_> = match std::fs::read_dir(&path) {
            Ok(dir) => dir
                .into_iter()
                .filter_map(|f| f.ok())
                .map(|f| f.path())
                .collect(),
            Err(err) => {
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
            .filter_map(|path| {
                let name = path.file_stem()?.to_str()?.to_string();
                let shape = Arc::new(RwLock::new(DisplacementShape::new_uninit(
                    &dbg,
                    path.clone(),
                    None,
                    conf.model_scale,
                )));
                displacement_shapes.insert(name.clone(), shape.clone());
                Some((
                    name.clone(),
                    Arc::new(RwLock::new(CompartmentCache::new(
                        &dbg,
                        shape.clone(),
                        conf.cache_dir.clone().join(PathBuf::from("compartments")),
                        name.clone(),
                        conf.compartment_heel_steps.clone(),
                        conf.compartment_trim_steps.clone(),
                        conf.compartment_level_step_qnt,
                        Arc::clone(&thread_pool),
                    ))),
                ))
            })
            .collect();
        let damaged_compartments = pathes
            .iter()
            .filter(|path: &&PathBuf| path.file_name().is_some())
            .filter_map(|path| {
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
                    Some(conf.model_x),
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
                        conf.hull_heel_steps.clone(),
                        conf.hull_trim_steps.clone(),
                        conf.hull_draught_min,
                        conf.hull_draught_max,
                        conf.hull_draught_step,
                        Arc::clone(&thread_pool),
                    ))),
                ))
            })
            .collect();
        let model_cached = Self {
            dbg: dbg.clone(),
            ship_length_lbp: conf.ship_length_lbp,
            model_x: conf.model_x,
            draught_min: conf.draught_min,
            hull_draught_step: conf.hull_draught_step,
            bounds_level_step: conf.bounds_level_step,
            cache_dir: conf.cache_dir.clone(),
            dso_angles: conf.dso_angles.clone(),
            displacement_shapes,
            windage_shape,
            displacement: DisplacementCache::new(
                &dbg,
                displacement_shape.clone(),
                conf.cache_dir.clone(),
                conf.hull_heel_steps.clone(),
                conf.hull_trim_steps.clone(),
                conf.hull_draught_min,
                conf.hull_draught_max,
                conf.hull_draught_step,
                Arc::clone(&thread_pool),
            ),
            compartments,
            hold_compartments: IndexMap::new(),
            damaged_compartments,
            windage_area,
            displacement_bounded: IndexMap::new(),
            compartments_bounded: IndexMap::new(),
            hold_compartments_bounded: IndexMap::new(),
            thread_pool,
        };
        //   dbg!(model_cached.compartments.len());
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
            let thread_name = format!("{}.reload_shapes displacement_shape {name}", &self.dbg);
            //    log::trace!("Starting thread {thread_name}");
            let handle = scheduler
                .spawn_named(thread_name, move || {
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
            let thread_name = format!("{}.reload_shapes windage_shape", &self.dbg);
            //    log::trace!("Starting thread {thread_name}");
            let handle = scheduler
                .spawn_named(thread_name, move || {
                    let mut guard = shape.write();
                    task_results.push(guard.init());
                    Ok(())
                })
                .map_err(|err| {
                    error.pass_with("spawn task area_shape".to_string(), err.to_string())
                });
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => errors.push(err),
            };
        }
        for task in tasks {
            //   log::trace!("join thread {}", task.name());
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
    pub fn init(
        &mut self,
        compartments_max: HashMap<String, (Option<f64>, f64)>,
        bounds: &Bounds,
    ) -> Result<(), Error> {
        //    dbg!(self.dbg.clone(), "init");
        let error = Error::new(self.dbg.clone(), "init");
        self.displacement
            .init()
            .map_err(|err| error.pass_with("displacement.init".to_string(), err))?;
        for (name, compartment) in self.compartments.iter_mut() {
            let mut guard = compartment.write();
            guard
                .init()
                .map_err(|err| error.pass_with(format!("compartment:{name}.init"), err))?;
            let (level_max, volume_max) = compartments_max
                .get(name)
                .ok_or(error.err(format!("compartments_volume_max.get(&name) {name}")))?;
            guard
                .calc_coeff(*volume_max, *level_max)
                .map_err(|err| error.pass_with(format!("compartment:{name}.calc_coeff"), err))?;
        }
        /*     TODO - пока не используются, потом будет отдельный расчет
        for (name, damaged_compartment) in self.damaged_compartments.iter_mut() {
            damaged_compartment
                .write()
                .init()
                .map_err(|err| error.pass_with(format!("damaged_compartment:{name}.init"), err))?
        }*/
        self.windage_area
            .init()
            .map_err(|err| error.pass_with("displacement.init".to_string(), err))?;
        let bounds_qnt = bounds.len_qnt();
        let displacement_shape = self
            .displacement_shapes
            .get("hull")
            .ok_or(error.err("no displacement_shape"))?;
        let displacement_bound = DisplacementBoundCache::new(
            &self.dbg,
            displacement_shape.clone(),
            self.cache_dir.clone().join("disp_bounded"),
            self.bounds_level_step,
            self.model_x,
            bounds.clone(),
            Arc::clone(&self.thread_pool),
        );
        displacement_bound
            .init()
            .map_err(|err| error.pass_with("displacement_bound.init".to_string(), err))?;
        self.displacement_bounded
            .insert(bounds_qnt, Arc::new(RwLock::new(displacement_bound)));
        let mut cache_map = IndexMap::new();
        for (code, compartment) in &self.compartments {
            let mut compartment_bounded = compartment
                .read()
                .build_bounded(bounds.clone(), self.bounds_level_step)
                .map_err(|err| error.pass_with("compartment_bounded.build_bounded", err))?;
            compartment_bounded
                .init()
                .map_err(|err| error.pass_with("compartment_bounded.init", err))?;
            cache_map.insert(code.clone(), Arc::new(RwLock::new(compartment_bounded)));
        }
        self.compartments_bounded
            .insert(bounds.len_qnt(), cache_map);
        Ok(())
    }
    /// Пересчет композитных отсеков трюма. Каждый расчет список таких отсеков обновляется
    /// и пересчитывается. К имеющимся отсекам добавляются новые. Старые не удаляются.
    pub fn update_hold_compartments(
        &mut self,
        new_hold_compartments: &Vec<(String, Vec<String>)>,
    ) -> Result<(), Error> {
        //    dbg!(self.dbg.clone(), "update_hold_compartments");
        let error = Error::new(self.dbg.clone(), "update_hold_compartments");
        for (code, codes_array) in new_hold_compartments {
            if !self.hold_compartments.contains_key(code) {
                let compartments: Vec<_> = codes_array
                    .iter()
                    .filter_map(|code| self.compartments.get(code))
                    .map(Arc::clone)
                    .collect();
                let new_hold_compartment = Arc::new(RwLock::new(
                    HoldCompartmentCache::new(&self.dbg, code, compartments)
                        .map_err(|err| error.pass(err))?,
                ));
                self.hold_compartments
                    .insert(code.to_owned(), new_hold_compartment);
            }
            for (qnt_bounds, compartments_bounded) in self.compartments_bounded.iter() {
                let mut hold_compartments_bounded = if let Some(hold_compartments_bounded) =
                    self.hold_compartments_bounded.get(qnt_bounds)
                {
                    hold_compartments_bounded.to_owned()
                } else {
                    IndexMap::new()
                };
                if !hold_compartments_bounded.contains_key(code) {
                    let compartments_bounded: Vec<_> = codes_array
                        .iter()
                        .filter_map(|code| compartments_bounded.get(code))
                        .map(Arc::clone)
                        .collect();
                    let new_hold_compartment_bounded = Arc::new(RwLock::new(
                        HoldCompartmentBoundCache::new(&self.dbg, code, compartments_bounded),
                    ));
                    hold_compartments_bounded.insert(code.to_owned(), new_hold_compartment_bounded);
                }
                self.hold_compartments_bounded
                    .insert(*qnt_bounds, hold_compartments_bounded);
            }
        }
        Ok(())
    }
    ///
    /// Пересчет кэшей корпуса
    #[allow(dead_code)]
    pub fn rebuild_hull(&mut self, bounds: &Bounds) -> Result<(), Error> {
        log::info!("rebuild_hull begin");
        let error = Error::new(&self.dbg, "rebuild_hull");
        let mut errors = Vec::new();
        // Считаем кэши, они сами по себе многопоточны, поэтому делить на потоки нет смысла
        if let Err(error) = self.displacement.rebuild() {
            errors.push(("displacement".to_owned(), error));
        }
        let displacement_shape = self
            .displacement_shapes
            .get("hull")
            .ok_or(error.err("no displacement_shape"))?;
        let mut displacement_bound = DisplacementBoundCache::new(
            &self.dbg,
            displacement_shape.clone(),
            self.cache_dir.clone().join("disp_bounded"),
            self.bounds_level_step,
            self.model_x,
            bounds.clone(),
            Arc::clone(&self.thread_pool),
        );
        displacement_bound
            .rebuild()
            .map_err(|err| error.pass_with("displacement_bound.rebuild", err))?;
        self.displacement_bounded
            .insert(bounds.len_qnt(), Arc::new(RwLock::new(displacement_bound)));
        self.windage_area
            .rebuild(bounds, self.ship_length_lbp)
            .map_err(|err| error.pass_with("windage_area.rebuild", err))?;
        if !errors.is_empty() {
            return Err(error.pass_with(
                "rebuild_hull",
                errors.iter().fold(String::new(), |acc, (key, err)| {
                    format!("{acc}\n\tIn cache {:?} was error: {err}", key)
                }),
            ));
        }
        log::info!("rebuild_hull finish");
        Ok(())
    }
    ///  Пересчет кэшей отсеков
    #[allow(dead_code)]
    pub fn rebuild_compartments(
        &mut self,
        bounds: &Bounds,
        compartments_max: HashMap<String, (Option<f64>, f64)>,
    ) -> Result<(), Error> {
        let error: Error = Error::new(&self.dbg, "rebuild_compartments");
        let mut errors = Vec::new();
        let mut cache_map = IndexMap::new();
        for (name, compartment) in &mut self.compartments {
            //        println!("model_cached rebuild compartment:{name}");
            let mut guard = compartment.write();
            if let Err(error) = guard.rebuild() {
                errors.push((("compartment ".to_owned() + name), error));
            }
            //   }
            //  for (name, compartment) in &self.compartments {
            //      println!("model_cached build_bounded compartment:{code}");
            //        guard.init().map_err(|err| error.pass_with("compartment_bounded.build_bounded", err))?;
            let (level_max, volume_max) = compartments_max
                .get(name)
                .ok_or(error.err(format!("compartments_volume_max.get(&name) {name}")))?;
            guard
                .calc_coeff(*volume_max, *level_max)
                .map_err(|err| error.pass_with(format!("compartment:{name}.calc_coeff"), err))?;
            let mut compartment_bounded = guard
                .build_bounded(bounds.clone(), self.bounds_level_step)
                .map_err(|err| error.pass_with("compartment_bounded.build_bounded", err))?;
            compartment_bounded
                .rebuild()
                .map_err(|err| error.pass_with("compartment_bounded.rebuild", err))?;
            cache_map.insert(name.clone(), Arc::new(RwLock::new(compartment_bounded)));
        }
        self.compartments_bounded
            .insert(bounds.len_qnt(), cache_map);
        /*  TODO - пока не используются, потом будет отдельный расчет
         for (name, compartment) in &mut self.damaged_compartments {
            if let Err(error) = compartment.write().rebuild() {
                errors.push((("damaged_compartment ".to_owned() + name), error));
            }
        }*/
        if !errors.is_empty() {
            return Err(error.pass_with(
                "rebuild_compartments",
                errors.iter().fold(String::new(), |acc, (key, err)| {
                    format!("{acc}\n\tIn cache {:?} was error: {err}", key)
                }),
            ));
        }

        Ok(())
    }
    ///
    /// Пересчет кэшей боковой поверхности корпуса
    #[allow(dead_code)]
    pub fn rebuild_windage(&mut self, bounds: &Bounds) -> Result<(), Error> {
        let error: Error = Error::new(&self.dbg, "rebuild_windage");
        self.windage_area
            .rebuild(bounds, self.ship_length_lbp)
            .map_err(|err| error.pass_with("windage_area.rebuild", err))?;
        Ok(())
    }
    ///  Пересчет кэшей без шпаций
    #[allow(dead_code)]
    pub fn rebuild_caches(&mut self) -> Result<(), Error> {
        log::info!("rebuild_caches begin");
        let error = Error::new(&self.dbg, "rebuild_caches");
        let mut errors = Vec::new();
        // Считаем кэши, они сами по себе многопоточны, поэтому делить на потоки нет смысла
        if let Err(error) = self.displacement.rebuild() {
            errors.push(("displacement".to_owned(), error));
        }
        for (name, compartment) in &mut self.compartments {
            //        println!("model_cached rebuild compartment:{name}");
            if let Err(error) = compartment.write().rebuild() {
                errors.push((("compartment ".to_owned() + name), error));
            }
        }
        /*  TODO - пока не используются, потом будет отдельный расчет
         for (name, compartment) in &mut self.damaged_compartments {
            if let Err(error) = compartment.write().rebuild() {
                errors.push((("damaged_compartment ".to_owned() + name), error));
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
    ///   Пересчет кэшей со шпациями
    #[allow(dead_code)]
    pub fn rebuild_bounds(&mut self, bounds: &Bounds) -> Result<(), Error> {
        let error: Error = Error::new(&self.dbg, "rebuild_bounds");
        let displacement_shape = self
            .displacement_shapes
            .get("hull")
            .ok_or(error.err("no displacement_shape"))?;
        let mut displacement_bound = DisplacementBoundCache::new(
            &self.dbg,
            displacement_shape.clone(),
            self.cache_dir.clone().join("disp_bounded"),
            self.bounds_level_step,
            self.model_x,
            bounds.clone(),
            Arc::clone(&self.thread_pool),
        );
        displacement_bound
            .rebuild()
            .map_err(|err| error.pass_with("displacement_bound.rebuild", err))?;
        self.displacement_bounded
            .insert(bounds.len_qnt(), Arc::new(RwLock::new(displacement_bound)));
        self.windage_area
            .rebuild(bounds, self.ship_length_lbp)
            .map_err(|err| error.pass_with("windage_area.rebuild", err))?;
        let mut cache_map = IndexMap::new();
        for (code, compartment) in &self.compartments {
            //      println!("model_cached build_bounded compartment:{code}");
            let mut compartment_bounded = compartment
                .read()
                .build_bounded(bounds.clone(), self.bounds_level_step)
                .map_err(|err| error.pass_with("compartment_bounded.build_bounded", err))?;
            compartment_bounded
                .rebuild()
                .map_err(|err| error.pass_with("compartment_bounded.rebuild", err))?;
            cache_map.insert(code.clone(), Arc::new(RwLock::new(compartment_bounded)));
        }
        self.compartments_bounded
            .insert(bounds.len_qnt(), cache_map);
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
    pub fn bounded_windage_area(&self) -> Result<Vec<f64>, Error> {
        self.windage_area.bounded_windage_area().map_err(|err| {
            Error::new(&self.dbg, "bounded_windage_area")
                .pass_with("self.windage_area.bounded_windage_area", err)
        })
    }
    /// Расчет параметров поверхности
    pub fn windage_area(&self, draught: f64) -> Result<AreaResult, Error> {
        self.windage_area.windage_area(draught).map_err(|err| {
            Error::new(&self.dbg, "windage_area").pass_with("self.windage_area.windage_area", err)
        })
    }
    /// Расчет параметров поверхности для минимальной осадки
    pub fn windage_area_min(&self) -> Result<(f64, f64, f64), Error> {
        self.windage_area.windage_area_min().map_err(|err| {
            Error::new(&self.dbg, "windage_area_min")
                .pass_with("self.windage_area.windage_area_min", err)
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
        let hold_compartments_bounded = self
            .hold_compartments_bounded
            .get(&query.bounds.len_qnt())
            .ok_or(error.err("no hold_compartments_bounded"))?;
        let scheduler = self.thread_pool.scheduler();
        // расчет эпюр масс для газообразных грузов
        // они не смещаются, поэтому считаем их один раз
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let gaseous_results = Arc::new(Stack::new());
        for cargo in query.gaseous {
            assert!(cargo.mass > 0.);
            let assigment_type = cargo.assigment_type;
            let code = cargo.code.clone();
            let error_ = error.err(format!("compartment_{code} gaseous work"));
            let compartments_bounded = compartments_bounded.clone();
            let results_ = gaseous_results.clone();
            let thread_name = format!("{}.balance_strength gaseous code:{}", &self.dbg, cargo.code);
            //    log::trace!("Starting thread {thread_name}");
            let handle = scheduler
                .spawn_named(thread_name, move || {
                    let compartment_bounded = compartments_bounded
                        .get(&code)
                        .ok_or(error_.err(format!("compartments_bounded.get no code:{code}")))?
                        .read();
                    let volume_bounded = compartment_bounded.get_max_volume().map_err(|err| {
                        error_.pass_with(format!("volume_bounded get_max, code:{code}"), err)
                    })?;
                    let volume: f64 = volume_bounded.iter().sum();
                    let density = if volume > 0. { cargo.mass / volume } else { 0. };
                    results_.push(balance::gaseous_result::GaseousResult::new(
                        code,
                        assigment_type,
                        volume_bounded.into_iter().map(|v| v * density).collect(),
                    ));
                    Ok(())
                })
                .map_err(|err| error.pass_with("scheduler.spawn".to_string(), err.to_string()));
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => errors.push(err),
            };
        }
        // расчет эпюр масс для сыпучих грузов
        // они не смещаются, поэтому считаем их один раз
        let bulk_results = Arc::new(Stack::new());
        for cargo in &query.bulk {
            assert!(cargo.mass > 0.);
            let assigment_type = cargo.assigment_type;
            let code = cargo.code.clone();
            let error_ = error.err(format!("compartment_{code} bulk work"));
            let density = cargo.mass / cargo.volume;
            let compartments_bounded = compartments_bounded.clone();
            let hold_compartments_bounded = hold_compartments_bounded.clone();
            let volume = cargo.volume;
            let epsilon = query.epsilon;
            let results_ = bulk_results.clone();
            let thread_name = format!("{}.balance_strength bulk code:{}", &self.dbg, cargo.code);
            //    log::trace!("Starting thread {thread_name}");
            let handle = scheduler
                .spawn_named(thread_name, move || {
                    let volume_bounded = if let Some(compartment) =
                        hold_compartments_bounded.get(&code)
                    {
                        compartment.read().get(volume, 0., epsilon).map_err(|err| {
                            error_.pass_with(format!("compartment_bounded.get, code:{code}"), err)
                        })?
                    } else {
                        let compartment_bounded = compartments_bounded
                            .get(&code)
                            .ok_or(error_.err(format!("compartments_bounded.get no code:{code}")))?
                            .read();
                        compartment_bounded
                            .get_from_volume(volume, 0., epsilon)
                            .map_err(|err| {
                                error_
                                    .pass_with(format!("compartment_bounded.get, code:{code}"), err)
                            })?
                    };
                    //    println!("model_cached balance_strength bulk code:{code} volume:{volume} volume_sum:{}", volume_bounded.iter().sum::<f64>());
                    results_.push(balance::bulk_result::BulkResult::new(
                        code,
                        assigment_type,
                        volume_bounded.into_iter().map(|v| v * density).collect(),
                    ));
                    Ok(())
                })
                .map_err(|err| error.pass_with("scheduler.spawn".to_string(), err.to_string()));
            match handle {
                Ok(task) => tasks.push(task),
                Err(err) => errors.push(err),
            };
        }
        for task in tasks {
            //    log::trace!("join thread {}", task.name());
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
                        error.pass_with("mass_distr.add_vec(gaseous)".to_string(), err.to_string())
                    })?;
                    gaseous.push(data);
                }
            }
            while !bulk_results.is_empty() {
                if let Some(data) = bulk_results.pop() {
                    res.add_vec(&data.mass_values).map_err(|err| {
                        error.pass_with("mass_distr.add_vec(bulk)", err.to_string())
                    })?;
                    bulk.push(data);
                }
            }
            res
        };
        let mut res_mass_distr = Vec::new();
        let mut res_displacement_distr = Vec::new();
        let (mut trim, mut draught) = (query.trim, query.draught);
        let (mut mass_sum, mut disp_sum) = (100000., 100000.);
        let mut epsilon_mass = 10.;
        let mut epsilon_x;
        for _i in 0..50 {
            // trim
            for _j in 0..50 {
                // draught
                let mut tasks: Vec<JoinHandle<_>> = vec![];
                let liquid_results = Arc::new(Stack::new());
                // жидкие грузы смещаются под действием силы тяжести
                for cargo in &query.liquid {
                    if cargo.mass == 0. {
                        continue;
                    }
                    let assigment_type = cargo.assigment_type;
                    let code = cargo.code.clone();
                    let cargo_type = cargo.cargo_type;
                    let error_ = error.err(format!("compartment_{code} liquid work"));
                    let density = cargo.mass / cargo.volume;
                    let compartment_bounded = compartments_bounded
                        .get(&code)
                        .ok_or(error_.err(format!("compartments_bounded.get no code:{code}")))?
                        .clone();
                    let trim = trim;
                    let volume = cargo.volume;
                    let epsilon = volume * epsilon_mass / mass_sum;
                    let results_ = liquid_results.clone();
                    let thread_name =
                        format!("{}.balance_strength liquid code:{}", &self.dbg, cargo.code);
                    //     log::trace!("Starting thread {thread_name}");
                    let handle = scheduler
                        .spawn_named(thread_name, move || {
                            let volume_bounded = compartment_bounded
                                .read()
                                .get_from_volume(volume, trim, epsilon)
                                .map_err(|err| {
                                    error_.pass_with(
                                        format!("compartment_bounded.get, code:{code}"),
                                        err,
                                    )
                                })?;
                            //         println!("model_cached code:{code} volume:{volume} volume_sum:{}", volume_bounded.iter().sum::<f64>());
                            results_.push(balance::liquid_result::LiquidResult::new(
                                code,
                                assigment_type,
                                cargo_type,
                                volume_bounded.into_iter().map(|v| v * density).collect(),
                            ));
                            Ok(())
                        })
                        .map_err(|err| {
                            error.pass_with("scheduler.spawn".to_string(), err.to_string())
                        });
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => errors.push(err),
                    };
                }
                let hull_results = Arc::new(Stack::new());
                let results_ = hull_results.clone();
                let displacement_bounded = displacement_bounded.clone();
                let thread_name = format!("{}.balance_strength displacement_bounded", &self.dbg);
                //    log::trace!("Starting thread {thread_name}");
                let handle = scheduler
                    .spawn_named(thread_name, move || {
                        results_.push(displacement_bounded.read().get(trim, draught));
                        Ok(())
                    })
                    .map_err(|err| {
                        error.pass_with(
                            "scheduler.spawn displacement_bounded".to_string(),
                            err.to_string(),
                        )
                    });
                match handle {
                    Ok(task) => tasks.push(task),
                    Err(err) => errors.push(err),
                };
                for task in tasks {
                    //     log::trace!("join thread {}", task.name());
                    if let Err(err) = task.join() {
                        let error = error.pass_with("task join", err.to_string());
                        log::error!("{}", error);
                        errors.push(error);
                    }
                }
                res_mass_distr = src_mass_distr.clone();
                liquid.clear();
                while !liquid_results.is_empty() {
                    if let Some(data) = liquid_results.pop() {
                        res_mass_distr.add_vec(&data.mass_values).map_err(|err| {
                            error.pass_with("mass_distr.add_vec(liquid)", err.to_string())
                        })?;
                        liquid.push(data);
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
                if delta_w.abs() <= epsilon_mass {
                    //      println!("bfgsdb break draught: {_j}, {epsilon_mass}");//, {draught}, {delta_w}, {mass_sum}, {disp_sum}");
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
            //         println!("bfgsdb trim: {_i}, {epsilon_mass}, {delta_x}");//, {trim}, {mass_x}, {disp_x}");
            if delta_x.abs() <= query.epsilon && epsilon_mass <= query.epsilon {
                //         println!("bfgsdb break trim: {_i}, {epsilon_mass}, {delta_x}, {trim}, {mass_x}, {disp_x}");
                break;
            }
            epsilon_x = delta_x.abs();
            epsilon_mass = (epsilon_x * 10.).max(query.epsilon);
            trim += delta_x / 5.;
        }
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
        query: &BalanceStabilityQuery,
        epsilon: f64,
    ) -> Result<BalanceStabilityResult, Error> {
        //   let time = std::time::Instant::now();
        let error = Error::new(&self.dbg, "balance_stability");
        let FloatingPositionResult {
            heel,
            trim,
            draught_mid,
            displacement,
            displacement_center,
            mass,
            mass_center,
            area_wl,
            area_wl_center,
            length_wl,
            breadth_wl,
            rad_long,
            rad_trans,
            ..
        } = self
            .floating_position(FloatingPositionQuery {
                water_density: query.water_density,
                mass_const: query.mass_const,
                moment_const: query.moment_const,
                bulk: query.bulk.clone(),
                liquid: query.liquid.clone(),
                damaged_compartment: Vec::new(),
                epsilon,
            })
            .map_err(|err| error.pass_with("self.floating_position", err))?;
        // println!("steps:{_i} time:{:?}", time.elapsed());
        let (draught_bow, draught_stern, draught_mean) = Draught::new(
            self.model_x,
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
        let liquid = self
            .process_liquid(&query.liquid, heel, trim, epsilon)
            .map_err(|err| error.pass(err))?;
        let bulk = self
            .process_bulk(&query.bulk, epsilon)
            .map_err(|err| error.pass(err))?;
        let bow_area = self
            .windage_area
            .bow_area(trim_degree, draught_mid)
            .map_err(|err| error.pass(err))?;
        /*    log::debug!(
            "ModelCached balance_stability bow_area:{bow_area}\nliquid:\n{}\nbulk:\n{}\n",
            liquid.iter().fold(String::new(), |s, v| s + &format!(
                "assignment_id:{} trans_moment_of_inertia:{:.3}\n",
                v.assignment_id, v.trans_moment_of_inertia
            )),
            bulk.iter().fold(String::new(), |s, v| s + &format!(
                "{} {}\n",
                v.assignment_id, v.mass_shift.print()
            )),
        );*/
        Ok(BalanceStabilityResult {
            heel,
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
            mass,
            mass_center,
            bow_area,
        })
    }
    /// Расчет [равновесного положения](https://github.com/a-givertzman/sss/blob/master/design/algorithm-simply/part03_draft/chapter01_floatingPosition/chapter01_floatingPosition.md)
    pub(crate) fn floating_position(
        &self,
        query: FloatingPositionQuery,
    ) -> Result<FloatingPositionResult, Error> {
        let error = Error::new(&self.dbg, "floating_position");
        if query.water_density <= 0. {
            return Err(error.err("water_density <= 0."));
        }
        let (min_volume, max_volume) = self
            .displacement
            .get_volume_disp()
            .map_err(|err| error.pass_with("self.get_volume_disp", err))?;
        // Считаем сыпучие грузы.
        // На них крен и дифферент не влияет.
        let moment_bulk = self
            .moment_bulk_floating(&query.bulk, query.epsilon)
            .map_err(|err| error.pass_with("self.bulk_moment", err))?;
        let mass_bulk = query.bulk.iter().map(|v| v.mass).sum::<f64>();
        //       dbg!(mass_bulk, moment_bulk.to_pos(mass_bulk));
        let mass_liquid = query.liquid.iter().map(|v| v.mass).sum::<f64>();
        let mass_sum = query.mass_const + mass_bulk + mass_liquid; // постоянная масса
        let moment_sum = query.moment_const + moment_bulk; // постоянный момент
        let volume = mass_sum / query.water_density;
        // допустимый объем корпуса
        if volume <= min_volume || volume >= max_volume {
            return Err(error.err(format!("volume <= min_volume || volume >= max_volume, mass:{mass_sum} min_volume:{min_volume} max_volume:{max_volume} water_density:{}", query.water_density)));
        }
        let mut heel = 0.0;
        let mut trim = 0.0;
        let mut draught = self.draught_min;
        let mut step_trim = 0.5_f64;
        let mut step_heel = 1.0_f64;
        let mut d_v: Option<f64> = None;
        let mut d_m: Option<f64> = None;
        for _i in 0..100 {
            let epsilon = (step_trim.max(step_heel)) / 10.;
            // учет смещения жидкости
            let moment_liquid = self
                .moment_liquid_floating(&query.liquid, heel, trim, epsilon)
                .map_err(|err| error.pass_with("self.moment_liquid_floating", err))?;
            let (new_draught, new_d_v, new_d_m, mass_center, displacement, disp_result) = self
                .position(
                    heel,
                    trim,
                    draught,
                    query.water_density,
                    epsilon,
                    mass_sum,
                    moment_sum,
                    moment_liquid,
                    &query.damaged_compartment,
                )
                .map_err(|err| error.pass(err))?;
            if query.epsilon >= epsilon {
                let precision = (new_d_v.powi(2) + new_d_m.powi(2)).sqrt();
                if precision < query.epsilon {
                    log::debug!(
                        "ModelCached floating position heel:{} trim:{} draught_mid:{} 
                        displacement:{} displacement_center:{}
                        mass_sum:{:.3} mass_center:{}
                        mass_bulk:{:.3} bulk_center:{}
                        mass_liquid:{:.3} liquid_center:{}
                        area_wl:{:.3} area_wl_center:{}
                        length_wl:{:.3} breadth_wl:{:.3}
                        inertia_long_y:{:.3}, inertia_trans_x:{:.3},
                        rad_long:{:.3} rad_trans:{:.3}",
                        heel,
                        trim,
                        new_draught,
                        displacement,
                        disp_result.volume_center.print(),
                        mass_sum,
                        mass_center.print(),
                        mass_bulk,
                        moment_bulk.to_pos(mass_bulk).print(),
                        mass_liquid,
                        moment_liquid.to_pos(mass_liquid).print(),
                        disp_result.area_wl,
                        disp_result.area_wl_center.print(),
                        disp_result.length_wl,
                        disp_result.breadth_wl,
                        disp_result.inertia_long_y,
                        disp_result.inertia_trans_x,
                        disp_result.inertia_long_y / displacement,
                        disp_result.inertia_trans_x / displacement,
                    );
                    let result = FloatingPositionResult {
                        heel,
                        trim,
                        draught_mid: new_draught,
                        precision,
                        displacement,
                        displacement_center: disp_result.volume_center,
                        mass: mass_sum,
                        mass_center,
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
            if let Some(old_d_v) = d_v
                && old_d_v.signum() != new_d_v.signum()
            {
                step_trim *= 0.1;
                step_heel *= 3.;
            }
            d_v = Some(new_d_v);
            if let Some(old_d_m) = d_m
                && old_d_m.signum() != new_d_m.signum()
            {
                step_heel *= 0.1;
                step_trim *= 3.;
            }
            d_m = Some(new_d_m);
            //        println!("hdghdfgdvb model_cached floating_position: {_i}, epsilon:{} h:{:.3}, t:{:.3}, draught:{:.3}, d_v:{}, d_m:{}",
            //            epsilon, heel, trim, draught, new_d_v, new_d_m);
            trim += step_trim * new_d_v.signum();
            heel += step_heel * new_d_m.signum();
            draught = new_draught;
        }
        Err(error.err(format!("query:{:?} error: no result", query)))
    }
    /// Расчет dso
    pub fn dso(
        &self,
        heel: f64,
        trim: f64,
        draught_mid: f64,
        cg: Position,
        query: BalanceStabilityQuery,
        opening: &[Position],
        deck_angle_point: &[Position],
        epsilon: f64,
    ) -> Result<DsoResult, Error> {
        //   let time = std::time::Instant::now();
        let error = Error::new(&self.dbg, "dso");
        let (dso, entry_angle, flooding_angle) = self
            .dso_surface_moment(
                query,
                heel,
                trim,
                draught_mid,
                epsilon,
                cg,
                &self.dso_angles,
                opening,
                deck_angle_point,
            )
            .map_err(|err| error.pass(err))?;
        Ok(DsoResult {
            dso,
            entry_angle,
            flooding_angle,
            heel,
        })
    }
    /// Расчет диаграммы статической остойчивости
    /// на основе фактического кренящего момента.
    /// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter02_bigAngles/deltaL.md]
    /// angles - углы крена должны быть отсортированны по возрастанию
    /// возвращает (dso, entry_angle, flooding_angle) зависимости от угла крена
    pub(crate) fn dso_abs_moment(
        &self,
        query: BalanceStabilityQuery,
        balanced_heel: f64,
        balanced_trim: f64,
        balanced_draught: f64,
        epsilon: f64,
        cg: Position,
        angles: &[f64],
        opening: &[Position],
        deck_angle_point: &[Position],
    ) -> Result<(Vec<(f64, f64)>, Vec<(f64, f64)>, Vec<(f64, f64)>), Error> {
        let error = Error::new(&self.dbg, "floating_position");
        if query.water_density <= 0. {
            return Err(error.err("water_density <= 0."));
        }
        // Считаем сыпучие грузы.
        // На них крен и дифферент не влияет.
        let moment_bulk = self
            .moment_bulk_floating(&query.bulk, epsilon)
            .map_err(|err| error.pass_with("self.bulk_moment", err))?;
        let mass_bulk = query.bulk.iter().map(|v| v.mass).sum::<f64>();
        let mass_liquid = query.liquid.iter().map(|v| v.mass).sum::<f64>();
        let mut trim = balanced_trim;
        let mut draught = balanced_draught;
        let mut step_trim = 0.1;
        let mut dso = Vec::new();
        let mut entry_angle = Vec::new();
        let mut flooding_angle = Vec::new();
        let mut last_heel: Option<f64> = None;
        let max_heel = angles.last().ok_or(error.err("max_heel"))?;
        let mass_sum = query.mass_const + mass_bulk + mass_liquid; // постоянная масса
        let moment_sum = query.moment_const + moment_bulk; // постоянный момент
        // println!("heel:yg:yc:ctg_phy:zg:zc:sqrt_v:res:");
        //  println!("model_cached dso heel trim moment_liquid delta_moment_liquid delta_l lv l");
        for &heel in angles {
            //       println!("\nmodel_cached dso heel:{heel} epsilon:{epsilon} step_trim:{}", step_trim);
            step_trim = if let Some(last_heel) = last_heel {
                step_trim * ((heel - last_heel) * 10.).max(1.)
            } else {
                0.1
            };
            last_heel = Some(heel);
            let mut d_v: Option<f64> = None;
            //      println!("model_cached dso lv:");
            for _i in 1..=100 {
                let trim_epsilon = step_trim / 10.;
                let moment_liquid_floating = self
                    .moment_liquid_floating(&query.liquid, heel, trim, epsilon)
                    .map_err(|err| error.pass_with("self.moment_liquid_dso", err))?;
                let (new_draught, new_d_v, _, _, _, disp_result) = self
                    .position(
                        heel,
                        trim,
                        draught,
                        query.water_density,
                        epsilon,
                        mass_sum,
                        moment_sum,
                        moment_liquid_floating,
                        &query.damaged_compartment,
                    )
                    .map_err(|err| error.pass(err))?;
                //   println!("sdffsz model_cached dso heel:{heel} i:{_i}, epsilon:{epsilon} trim_epsilon:{trim_epsilon} d_v:{new_d_v}");
                if epsilon >= trim_epsilon && epsilon >= new_d_v.abs() {
                    let delta_moment_liquid = self
                        .moment_liquid_dso_abs_moment(
                            &query.liquid,
                            heel,
                            trim,
                            balanced_heel,
                            balanced_trim,
                            epsilon,
                        )
                        .map_err(|err| error.pass_with("self.moment_liquid", err))?;
                    let l = {
                        let [_, tcg, vcg] = cg.values();
                        let [_, tcb, vcb] = disp_result.volume_center.values();
                        let sin_theta = heel.to_radians().sin();
                        let cos_theta = heel.to_radians().cos();
                        let sin_delta_angle = (heel - balanced_heel).to_radians().sin();
                        let lv = tcb * cos_theta + vcb * sin_theta;
                        //             println!("{heel} {lv};");
                        let ld = tcg * cos_theta + vcg * sin_theta;
                        let delta_l = delta_moment_liquid * sin_delta_angle / mass_sum;
                        let l = lv - ld - delta_l;
                        //   println!("model_cached dso heel:{heel} trim:{trim} moment_liquid_dso:{moment_liquid_dso} delta_moment_liquid:{delta_moment_liquid} delta_l:{delta_l} lv:{lv} l:{l}");
                        println!("{heel} {trim} {delta_moment_liquid} {delta_l} {lv} {l};");
                        l
                    };
                    dso.push((heel, l));
                    let tg_t = trim.to_radians().tan();
                    let tg_h = heel.to_radians().tan();
                    let cos_h = heel.to_radians().cos();
                    let current_draught = |p: &Position| {
                        let d_zi = p.y() * tg_h + (p.x() - self.model_x) * tg_t / cos_h;
                        p.z() - draught - d_zi
                    };
                    let min_angle = |angles: &[Position]| {
                        let mut angles: Vec<_> = angles.iter().map(&current_draught).collect();
                        angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        angles.first().unwrap_or(max_heel).to_owned()
                    };
                    entry_angle.push((heel, min_angle(opening)));
                    flooding_angle.push((heel, min_angle(deck_angle_point)));
                    break;
                }
                if let Some(old_d_v) = d_v
                    && old_d_v.signum() != new_d_v.signum()
                {
                    step_trim *= 0.5;
                }
                d_v = Some(new_d_v);
                trim += step_trim * new_d_v.signum();
                draught = new_draught;
            }
        }
        /*       println!("\nmodel_cached dso: ");
        for &(angle, value) in dso.iter() {
            println!("{angle} {value};");
        }
             println!("\nmodel_cached entry_angle: ");
        for &(angle, value) in entry_angle.iter() {
            println!("{angle} {value};");
        }
        println!("\nmodel_cached flooding_angle: ");
        for &(angle, value) in flooding_angle.iter() {
            println!("{angle} {value};");
        }*/
        Ok((dso, entry_angle, flooding_angle))
    }
    /// Расчет диаграммы статической остойчивости
    /// на основе поперечного момента инерции площади ватерлинии.
    /// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter02_bigAngles/deltaL.md]
    /// angles - углы крена должны быть отсортированны по возрастанию
    /// возвращает (dso, entry_angle, flooding_angle) зависимости от угла крена
    pub(crate) fn dso_surface_moment(
        &self,
        query: BalanceStabilityQuery,
        balanced_heel: f64,
        balanced_trim: f64,
        balanced_draught: f64,
        epsilon: f64,
        cg: Position,
        angles: &[f64],
        opening: &[Position],
        deck_angle_point: &[Position],
    ) -> Result<(Vec<(f64, f64)>, Vec<(f64, f64)>, Vec<(f64, f64)>), Error> {
        let error = Error::new(&self.dbg, "floating_position");
        if query.water_density <= 0. {
            return Err(error.err("water_density <= 0."));
        }
        // Считаем сыпучие грузы.
        // На них крен и дифферент не влияет.
        let moment_bulk = self
            .moment_bulk_floating(&query.bulk, epsilon)
            .map_err(|err| error.pass_with("self.bulk_moment", err))?;
        let mass_bulk = query.bulk.iter().map(|v| v.mass).sum::<f64>();
        let mass_liquid = query.liquid.iter().map(|v| v.mass).sum::<f64>();
        let mut trim = balanced_trim;
        let mut draught = balanced_draught;
        let mut step_trim = 0.1;
        let mut dso = Vec::new();
        let mut entry_angle = Vec::new();
        let mut flooding_angle = Vec::new();
        let mut last_heel: Option<f64> = None;
        let max_heel = angles.last().ok_or(error.err("max_heel"))?;
        let mass_sum = query.mass_const + mass_bulk + mass_liquid; // постоянная масса
        let moment_sum = query.moment_const + moment_bulk; // постоянный момент
        let moment_liquid_surface =
            self // момент инерции площади ватерлинии жидкости
                .moment_liquid_dso_surface_moment(&query.liquid, epsilon)
                .map_err(|err| error.pass_with("self.moment_liquid", err))?;
        // println!("model_cached dso_surface_moment mass_bulk:{:.3} moment_bulk:{} mass_liquid:{:.3} mass_sum:{:.3} moment_liquid_surface:{:.3}",
        //     mass_bulk, moment_bulk.to_pos(mass_bulk).print(), mass_liquid, mass_sum, moment_liquid_surface);
        // println!("heel:yg:yc:ctg_phy:zg:zc:sqrt_v:res:");
        //  println!("model_cached dso heel trim moment_liquid delta_moment_liquid delta_l lv l");
        for &heel in angles {
            //       println!("\nmodel_cached dso heel:{heel} epsilon:{epsilon} step_trim:{}", step_trim);
            step_trim = if let Some(last_heel) = last_heel {
                step_trim * ((heel - last_heel) * 10.).max(1.)
            } else {
                0.1
            };
            last_heel = Some(heel);
            let mut d_v: Option<f64> = None;
            //      println!("model_cached dso lv:");
            for _i in 1..=100 {
                let trim_epsilon = step_trim / 10.;
                let moment_liquid_floating = self
                    .moment_liquid_floating(&query.liquid, heel, trim, epsilon)
                    .map_err(|err| error.pass_with("self.moment_liquid_dso", err))?;
                let (new_draught, new_d_v, _, _, _, disp_result) = self
                    .position(
                        heel,
                        trim,
                        draught,
                        query.water_density,
                        epsilon,
                        mass_sum,
                        moment_sum,
                        moment_liquid_floating,
                        &query.damaged_compartment,
                    )
                    .map_err(|err| error.pass(err))?;
                //       println!("sdffsz model_cached dso heel:{:.3} i:{_i} shift_liquid:{}, trim:{:.3} step_trim:{:.3} epsilon:{:.3} trim_epsilon:{:.3} d_v:{:.3} new_d_v:{:.3} ",
                //          heel, moment_liquid_floating.to_pos(mass_liquid).print(), trim, step_trim, epsilon, trim_epsilon, d_v.unwrap_or(0.), new_d_v);
                if epsilon >= trim_epsilon && epsilon >= new_d_v.abs() {
                    let l = {
                        let [_, tcg, vcg] = cg.values();
                        let [_, tcb, vcb] = disp_result.volume_center.values();
                        let sin_theta = heel.to_radians().sin();
                        let cos_theta = heel.to_radians().cos();
                        let sin_delta_angle = (heel - balanced_heel).to_radians().sin();
                        let lv = tcb * cos_theta + vcb * sin_theta;
                        //             println!("{heel} {lv};");
                        let ld = tcg * cos_theta + vcg * sin_theta;
                        let delta_l = moment_liquid_surface * sin_delta_angle / mass_sum;

                        //   println!("model_cached "heel:{heel} trim:{trim} shift_liquid:{} delta_l:{delta_l} lv:{lv} l:{l}", moment_liquid_floating.to_pos(mass_liquid).print(), );
                        //   println!("model_cached dso heel:{heel} trim:{trim} moment_liquid_dso:{moment_liquid_dso} delta_moment_liquid:{delta_moment_liquid} delta_l:{delta_l} lv:{lv} l:{l}");
                        //     println!("{heel} {trim} {moment_liquid_surface} {delta_l} {lv} {l};");
                        lv - ld - delta_l
                    };
                    dso.push((heel, l));
                    let tg_t = trim.to_radians().tan();
                    let tg_h = heel.to_radians().tan();
                    let cos_h = heel.to_radians().cos();
                    let current_draught = |p: &Position| {
                        let d_zi = p.y() * tg_h + (p.x() - self.model_x) * tg_t / cos_h;
                        p.z() - draught - d_zi
                    };
                    let min_angle = |angles: &[Position]| {
                        let mut angles: Vec<_> = angles.iter().map(&current_draught).collect();
                        angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        angles.first().unwrap_or(max_heel).to_owned()
                    };
                    flooding_angle.push((heel, min_angle(opening)));
                    entry_angle.push((heel, min_angle(deck_angle_point)));
                    break;
                }
                if let Some(old_d_v) = d_v
                    && old_d_v.signum() != new_d_v.signum()
                {
                    step_trim *= 0.5;
                }
                d_v = Some(new_d_v);
                trim += step_trim * new_d_v.signum();
                draught = new_draught;
            }
        }
        /*     println!("\nmodel_cached dso: ");
        for &(angle, value) in dso.iter() {
            println!("{angle} {value};");
        }
           println!("\nmodel_cached entry_angle: ");
        for &(angle, value) in entry_angle.iter() {
            println!("{angle} {value};");
        }
        println!("\nmodel_cached flooding_angle: ");
        for &(angle, value) in flooding_angle.iter() {
            println!("{angle} {value};");
        }*/
        Ok((dso, entry_angle, flooding_angle))
    }
    /// Расчет итерации в расчете [равновесного положения](https://github.com/a-givertzman/sss/blob/master/design/algorithm/part03_draft/chapter01_floatingPosition/chapter01_floatingPosition.md)
    /// и диаграммы. Возвращает (draught, d_v, d_m, cg, displacement, disp_result, mass_shift_z)
    fn position(
        &self,
        heel: f64,
        trim: f64,
        draught: f64,
        water_density: f64,
        epsilon: f64,
        mass_sum: f64,        // постоянная масса mass_const + mass_bulk + mass_liquid
        moment_sum: Position, // постоянный момент moment_const + moment_bulk
        moment_liquid: Moment,
        damaged_compartment: &Vec<String>,
    ) -> Result<(f64, f64, f64, Position, f64, DisplacementCacheResult), Error> {
        let error = Error::new(&self.dbg, "_floating_position");
        // учет изменения водоизмещения из-за поврежденных отсеков
        // поврежденные отсеки есть только в аварийном расчете, иначе список пустой
        let (mass_damaged_compartment, moment_damaged_compartment) = self
            .calc_damaged_compartments(damaged_compartment, heel, trim, draught, water_density)
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
        let cg: Position = {
            //        dbg!(moment_sum, moment_liquid, mass_sum);
            let moment_sum = moment_sum + moment_liquid + moment_damaged_compartment;
            moment_sum.to_pos(mass_sum)
        };
        //    dbg!(cg);
        // Определение невязки
        let cg_h = {
            // Через центр плавучести CB проводится горизонтальная плоскость
            let my_plane = HalfSpace::new(Vector3::z_axis());
            let cg_local = cg - cb;
            let cg_local = rotation.transform_point(&cg_local.into());
            let cg_h_local = my_plane.project_local_point(&cg_local, false).point;
            let cg_h = rotation.inverse_transform_point(&cg_h_local);

            cb + cg_h.into()
        };
        // Определение посадки судна для следующего шага
        let cb_v = {
            // Через центр плавучести CG проводится вертикальная плоскость параллельная основной линии
            let my_plane = HalfSpace::new(Vector3::y_axis());
            let cb_local = cb - cg;
            let cb_local = rotation.transform_point(&cb_local.into());
            let cg_v_local = my_plane.project_local_point(&cb_local, false).point;
            let cg_v = rotation.inverse_transform_point(&cg_v_local);

            cg + cg_v.into()
        };
        let cb_m = {
            // Через центр плавучести CG проводится вертикальная плоскость параллельная миделю
            let my_plane = HalfSpace::new(Vector3::x_axis());
            let cb_local = cb - cg;
            let cb_local = rotation.transform_point(&cb_local.into());
            let cb_m_local = my_plane.project_local_point(&cb_local, false).point;
            let cb_m = rotation.inverse_transform_point(&cb_m_local);

            cg + cb_m.into()
        };
        // проекция точки cg_m на вертикальную плоскость параллельную основной линии
        let cg_m_h = {
            // Через центр плавучести CG проводится вертикальная плоскость параллельная основной линии
            let my_plane = HalfSpace::new(Vector3::y_axis());
            let cg_m_local = cb_m - cg;
            let cg_m_local = rotation.transform_point(&cg_m_local.into());
            let cg_m_h_local = my_plane.project_local_point(&cg_m_local, false).point;
            let cg_m_h = rotation.inverse_transform_point(&cg_m_h_local);

            cg + cg_m_h.into()
        };
        let d_v = cg_h.x() - cb_v.x();
        let d_m = cg_m_h.y() - cb_m.y();
        //   println!("hdghdfgdvb model_cached position: heel:{:.3} trim:{:.3} draught:{:.3}  cg:{}, cb:{} cg_h:{} cb_v:{} cb_m:{} d_v:{:.3}, d_m:{:.3}",
        //          heel, trim, draught, cg.print(), cb.print(), cg_h.print(), cb_v.print(), cb_m.print(), d_v, d_m);
        //  println!("hdghdfgdvb model_cached position: heel:{} cg:{}, cb:{} cg_h:{} cb_v:{} d_m:{}",
        //          heel, cg.y(), cb.y(), cg_h.y(), cb_v.y(), d_m);
        Ok((draught, d_v, d_m, cg, displacement, disp_result))
    }
    // Считаем сыпучие грузы.
    // На них крен и дифферент не влияет.
    fn process_bulk(&self, bulks: &Vec<BulkData>, epsilon: f64) -> Result<Vec<BulkResult>, Error> {
        let error = Error::new(&self.dbg, "moment_bulk");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let scheduler = self.thread_pool.scheduler();
        for cargo in bulks {
            assert!(cargo.mass > 0.);
            let code = cargo.code.clone();
            let error_ = error.err(format!("compartment_{code} bulk work"));
            let cargo = cargo.clone();
            let results_ = task_results.clone();
            let thread_name = format!("{}.process_bulk code:{code}", &self.dbg);
            //    log::trace!("Starting thread {thread_name}");
            if let Some(hold_compartment) = self.hold_compartments.get(&code) {
                let hold_compartment = Arc::clone(hold_compartment);
                let handle = scheduler
                    .spawn_named(thread_name, move || {
                        let compartment_result = hold_compartment
                            .read()
                            .get_level(0., 0., cargo.volume, epsilon)
                            .map_err(|err| error_.pass_with("hold_compartment.get", err))?;
                        //         dbg!(&code, cargo.mass, compartment_result.volume_center);
                        results_.push(stability_result::BulkResult::new(
                            //       cargo_id,
                            code,
                            cargo.assignment_id,
                            cargo.assigment_type,
                            cargo.mass,
                            cargo.stowage_factor,
                            compartment_result.volume_center,
                            cargo.shiftable,
                            compartment_result.level,
                            compartment_result.volume,
                        ));
                        Ok(())
                    })
                    .map_err(|err| error.pass_with("scheduler.spawn".to_string(), err.to_string()));
                match handle {
                    Ok(task) => tasks.push(task),
                    Err(err) => errors.push(err),
                };
            } else {
                let compartment = Arc::clone(
                    self.compartments
                        .get(&code)
                        .ok_or(error.err(format!("no compartment:{code}")))?,
                );
                let handle = scheduler
                    .spawn_named(thread_name, move || {
                        let compartment_result = compartment
                            .read()
                            .get_level(0., 0., cargo.volume, epsilon)
                            .map_err(|err| error_.pass_with("compartment.get", err))?;
                        dbg!(&code, cargo.mass, compartment_result.volume_center);
                        results_.push(stability_result::BulkResult::new(
                            //       cargo_id,
                            code,
                            cargo.assignment_id,
                            cargo.assigment_type,
                            cargo.mass,
                            cargo.stowage_factor,
                            compartment_result.volume_center,
                            cargo.shiftable,
                            compartment_result.level,
                            compartment_result.volume,
                        ));
                        Ok(())
                    })
                    .map_err(|err| error.pass_with("scheduler.spawn".to_string(), err.to_string()));
                match handle {
                    Ok(task) => tasks.push(task),
                    Err(err) => errors.push(err),
                };
            }
        }
        for task in tasks {
            //    log::trace!("join thread {}", task.name());
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
        let mut result = Vec::new();
        while !task_results.is_empty() {
            if let Some(data) = task_results.pop() {
                result.push(data);
            }
        }
        Ok(result)
    }
    /// Считаем момент сыпучих грузов
    fn moment_bulk_floating(&self, bulks: &Vec<BulkData>, epsilon: f64) -> Result<Moment, Error> {
        let error = Error::new(&self.dbg, "moment_bulk_floating");
        let bulks = self
            .process_bulk(bulks, epsilon)
            .map_err(|err| error.pass_with("process_bulk", err))?;
        let (_sum_mass, sum_moment): (f64, Moment) =
            bulks
                .iter()
                .fold((0., Moment::zero()), |(mass_sum, moment_sum), v| {
                    (
                        mass_sum + v.mass,
                        moment_sum + Moment::from_pos(v.mass_shift, v.mass),
                    )
                });
        //    dbg!(_sum_mass, sum_moment);
        Ok(sum_moment)
    }
    /// Считаем жидкие грузы
    fn process_liquid(
        &self,
        liquids: &Vec<LiquidData>,
        heel: f64,
        trim: f64,
        epsilon: f64,
    ) -> Result<Vec<LiquidResult>, Error> {
        //     if heel == 0. {  println!("\n\nprocess_liquid heel:{} trim:{}\n", heel, trim,);  }
        let error = Error::new(&self.dbg, "process_liquid");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let scheduler = self.thread_pool.scheduler();
        for cargo in liquids {
            match self.compartments.get(&cargo.code) {
                Some(compartment) => {
                    let task_results = task_results.clone();
                    let epsilon = epsilon;
                    let code = cargo.code.clone();
                    let cargo = cargo.clone();
                    let error_ = error.clone();
                    let compartment = compartment.clone();
                    let thread_name = format!("{}.process_liquid code:{}", &self.dbg, code);
                    //    log::trace!("Starting thread {thread_name}");
                    let handle = scheduler
                        .spawn_named(thread_name, move || {
                            let res = compartment
                                .read()
                                .get_for_stability(
                                    heel,
                                    trim,
                                    cargo.volume,
                                    epsilon,
                                    cargo.use_max_moment,
                                    cargo.is_cargo_tank,
                                )
                                .map_err(|err| error_.pass_with("compartment.get", err))?;
                            task_results.push((
                                code.clone(),
                                stability_result::LiquidResult::new(
                                    code.clone(),
                                    cargo.assignment_id,
                                    cargo.assigment_type,
                                    cargo.mass,
                                    res.volume_center,
                                    res.level,
                                    res.volume,
                                    res.inertia_trans_x,
                                    res.inertia_long_y,
                                    res.max_inertia_trans_x,
                                ),
                            ));
                            Ok(())
                        })
                        .map_err(|err| error.pass_with(format!("spawn for {}", cargo.code), err));
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => {
                            let error = error.pass_with(format!("handle for {}", cargo.code), err);
                            log::error!("{}", error);
                            errors.push(error);
                        }
                    };
                }
                None => {
                    let error = error.err(format!("no compartment: {}", cargo.code));
                    log::error!("{}", error);
                    errors.push(error);
                }
            }
        }
        for task in tasks {
            //   log::trace!("join thread {}", task.name());
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(error);
            }
        }
        if !errors.is_empty() {
            return Err(error.pass_with(
                "moment_liquid",
                errors.iter().fold("errors:".to_string(), |acc, err| {
                    acc + &format!("\n{}", err)
                }),
            ));
        }
        let mut values = Vec::new();
        while !task_results.is_empty() {
            if let Some((_code, result)) = task_results.pop() {
                //        if heel == -20. && trim < 3. { println!("heel:{heel} trim:{trim} {} {} {};", _code, result.mass, result.mass_shift.print());  }
                values.push(result);
            }
        }
        Ok(values)
    }
    /// Считаем момент жидких грузов
    fn moment_liquid_floating(
        &self,
        liquids: &Vec<LiquidData>,
        heel: f64,
        trim: f64,
        epsilon: f64,
    ) -> Result<Moment, Error> {
        let error = Error::new(&self.dbg, "moment_liquid");
        let values = self
            .process_liquid(liquids, heel, trim, epsilon)
            .map_err(|err| error.pass_with("process_liquid", err))?;
        let (_, sum_moment): (f64, Moment) =
            values
                .iter()
                .fold((0., Moment::zero()), |(mass_sum, moment_sum), v| {
                    (
                        mass_sum + v.mass,
                        moment_sum + Moment::from_pos(v.mass_shift, v.mass),
                    )
                });
        //   dbg!(sum_mass, sum_moment);
        Ok(sum_moment)
    }
    /// Считаем момент жидких грузов для расчета ДСО
    /// на основе фактического кренящего момента.
    /// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter02_bigAngles/deltaL.md]
    /// объем меняется в зависимости от признаков и процента наполнения отсека
    fn moment_liquid_dso_abs_moment(
        &self,
        liquids: &Vec<LiquidData>,
        current_heel: f64,
        current_trim: f64,
        balanced_heel: f64,
        balanced_trim: f64,
        epsilon: f64,
    ) -> Result<f64, Error> {
        // if heel == 0.
        //  {  println!("\n\nmoment_liquid_dso heel:{} trim:{}\n", heel, trim,);  }
        let error = Error::new(&self.dbg, "moment_liquid_dso");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let mut values = Vec::new();
        let scheduler = self.thread_pool.scheduler();
        for cargo in liquids {
            match self.compartments.get(&cargo.code) {
                Some(compartment) => {
                    let task_results = task_results.clone();
                    let error_ = error.clone();
                    let code = cargo.code.clone();
                    let cargo = cargo.clone();
                    let compartment = compartment.clone();
                    let thread_name =
                        format!("{}.moment_liquid_dso_abs_moment code:{}", &self.dbg, code);
                    //    log::trace!("Starting thread {thread_name}");
                    let handle = scheduler
                        .spawn_named(thread_name, move || {
                            let res = compartment
                                .read()
                                .get_for_dso_abs_moment(
                                    current_heel,
                                    current_trim,
                                    cargo.volume,
                                    balanced_heel,
                                    balanced_trim,
                                    epsilon,
                                    cargo.use_max_moment,
                                    cargo.is_cargo_tank,
                                )
                                .map_err(|err| error_.pass_with("compartment.get", err))?;
                            task_results.push((code, cargo.density, res));
                            Ok(())
                        })
                        .map_err(|err| error.pass_with(format!("spawn for {}", cargo.code), err));
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => {
                            let error = error.pass_with(format!("handle for {}", cargo.code), err);
                            log::error!("{}", error);
                            println!("{}", error);
                            errors.push(error);
                        }
                    };
                }
                None => {
                    let error = error.err(format!("no compartment: {}", cargo.code));
                    log::error!("{}", error);
                    println!("{}", error);
                    errors.push(error);
                }
            }
        }
        for task in tasks {
            //   log::trace!("join thread {}", task.name());
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                println!("{}", error);
                errors.push(error);
            }
        }
        while !task_results.is_empty() {
            if let Some((_code, density, moment)) = task_results.pop() {
                //      if _code == "501" { println!("moment_liquid_dso heel:{heel} code:{} {} {} {};", _code, result.volume_center.y(), result.volume, result.volume_center.y() * result.volume * density);  }
                // println!("moment_liquid_dso heel:{heel} code:{} {} {};", _code, result.volume_center.y(), result.volume);
                /*          println!(
                    "moment_liquid_dso heel:{current_heel} code:{} {};",
                    _code,
                    moment
                );*/
                values.push(moment * density);
            }
        }
        if !errors.is_empty() {
            let error = error.pass_with(
                "moment_liquid",
                errors.iter().fold("errors:".to_string(), |acc, err| {
                    acc + &format!("\n{}", err)
                }),
            );
            log::error!("{}", error);
            println!("{}", error);
            return Err(error);
        }
        let sum_moment = values.into_iter().sum();
        //     println!("moment_liquid_dso sum_moment {heel} {sum_moment}");
        Ok(sum_moment)
    }
    /// Считаем момент жидких грузов для расчета ДСО
    /// на основе поперечного момента инерции площади ватерлинии.
    /// [https://github.com/a-givertzman/sss/blob/master/design/algorithm/part04_stability/chapter02_bigAngles/deltaL.md]
    /// объем меняется в зависимости от признаков и процента наполнения отсека
    fn moment_liquid_dso_surface_moment(
        &self,
        liquids: &Vec<LiquidData>,
        epsilon: f64,
    ) -> Result<f64, Error> {
        // if heel == 0.
        //  {  println!("\n\nmoment_liquid_dso heel:{} trim:{}\n", heel, trim,);  }
        let error = Error::new(&self.dbg, "moment_liquid_dso");
        let mut tasks: Vec<JoinHandle<_>> = vec![];
        let task_results = Arc::new(Stack::new());
        let mut errors = Vec::new();
        let mut values = Vec::new();
        let scheduler = self.thread_pool.scheduler();
        for cargo in liquids {
            match self.compartments.get(&cargo.code) {
                Some(compartment) => {
                    let task_results = task_results.clone();
                    let error_ = error.clone();
                    let code = cargo.code.clone();
                    let cargo = cargo.clone();
                    let compartment = compartment.clone();
                    let thread_name = format!(
                        "{}.moment_liquid_dso_surface_moment code:{}",
                        &self.dbg, code
                    );
                    //    log::trace!("Starting thread {thread_name}");
                    let handle = scheduler
                        .spawn_named(thread_name, move || {
                            let res = compartment
                                .read()
                                .get_for_dso_surface_moment(
                                    cargo.volume,
                                    epsilon,
                                    cargo.use_max_moment,
                                    cargo.is_cargo_tank,
                                )
                                .map_err(|err| error_.pass_with("compartment.get", err))?;
                            task_results.push((code, cargo.density, res));
                            Ok(())
                        })
                        .map_err(|err| error.pass_with(format!("spawn for {}", cargo.code), err));
                    match handle {
                        Ok(task) => tasks.push(task),
                        Err(err) => {
                            let error = error.pass_with(format!("handle for {}", cargo.code), err);
                            log::error!("{}", error);
                            println!("{}", error);
                            errors.push(error);
                        }
                    };
                }
                None => {
                    let error = error.err(format!("no compartment: {}", cargo.code));
                    log::error!("{}", error);
                    println!("{}", error);
                    errors.push(error);
                }
            }
        }
        for task in tasks {
            //    log::trace!("join thread {}", task.name());
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                println!("{}", error);
                errors.push(error);
            }
        }
        while !task_results.is_empty() {
            if let Some((_code, density, moment)) = task_results.pop() {
                values.push(moment * density);
            }
        }
        if !errors.is_empty() {
            let error = error.pass_with(
                "moment_liquid",
                errors.iter().fold("errors:".to_string(), |acc, err| {
                    acc + &format!("\n{}", err)
                }),
            );
            log::error!("{}", error);
            println!("{}", error);
            return Err(error);
        }
        let sum_moment = values.into_iter().sum();
        //     println!("moment_liquid_dso sum_moment {heel} {sum_moment}");
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
                    let code = damaged_compartment.clone();
                    let compartment = compartment.clone();
                    let _error = error.clone();
                    let thread_name =
                        format!("{}.calc_damaged_compartments code:{}", &self.dbg, code);
                    //    log::trace!("Starting thread {thread_name}");
                    let handle = scheduler
                        .spawn_named(thread_name, move || {
                            task_results.push((code, compartment.read().get(heel, trim, draught)));
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
            //    log::trace!("join thread {}", task.name());
            if let Err(err) = task.join() {
                let error = error.pass_with("task join", err.to_string());
                log::error!("{}", error);
                errors.push(error);
            }
        }
        while !task_results.is_empty() {
            if let Some((code, data)) = task_results.pop() {
                let (mass, position) = match data {
                    Ok((volume, position)) => (volume * water_density, position),
                    Err(err) => {
                        let error = error
                            .pass_with(format!("task_results data in {code}"), err.to_string());
                        log::error!("{}", error);
                        errors.push(error);
                        continue;
                    }
                };
                values.push((code, mass, Moment::from_pos(position, mass)));
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
//
#[cfg(test)]
impl ModelCached {
    /// Создает пустую заглушку ModelCached для тестов
    pub fn mock_empty() -> Self {
        let dbg = Dbg::new("test", "ModelCachedMock");

        // Создаем пустую форму парусности
        let windage_shape = Arc::new(RwLock::new(AreaShape::create_test_rectangle(100, 10, 1.)));

        Self {
            dbg: dbg.clone(),
            ship_length_lbp: 100.0,
            model_x: 0.0,
            draught_min: 1.0,
            hull_draught_step: 0.1,
            bounds_level_step: 0.1,
            cache_dir: PathBuf::from("/tmp"),
            dso_angles: vec![],
            displacement_shapes: IndexMap::new(),
            windage_shape,
            displacement: DisplacementCache::create_simple_mock(10000., 10000.),
            compartments: IndexMap::new(),
            hold_compartments: IndexMap::new(),
            damaged_compartments: IndexMap::new(),
            windage_area: WindageArea::create_simple_mock(Vec::new()),
            displacement_bounded: IndexMap::new(),
            compartments_bounded: IndexMap::new(),
            hold_compartments_bounded: IndexMap::new(),
            thread_pool: Arc::new(ThreadPool::new("ModelCached::mock_empty", None)),
        }
    }
    /// Создает заглушку ModelCached для тестов прочности
    /// с предустановленным распределением парусности
    pub fn mock_with_strength_areas(windage_area_str: Vec<f64>) -> Self {
        let dbg = sal_core::dbg::Dbg::new("test", "ModelCachedMockStrength");

        // 1. Создаем пустую базовую форму парусности
        let windage_shape = Arc::new(RwLock::new(AreaShape::create_test_rectangle(100, 10, 1.0)));

        // 2. Используем ваш метод для создания мока WindageArea
        // Он запишет переданный вектор в поле values и создаст нужные пустые заглушки
        let mock_windage = WindageArea::create_simple_mock(windage_area_str);

        Self {
            dbg: dbg.clone(),
            ship_length_lbp: 100.0,
            model_x: 0.0,
            draught_min: 1.0,
            hull_draught_step: 0.1,
            bounds_level_step: 0.1,
            cache_dir: std::path::PathBuf::from("/tmp"),
            dso_angles: vec![],
            displacement_shapes: indexmap::IndexMap::new(),
            windage_shape,

            // Используем ваш простой мок гидростатики
            displacement: DisplacementCache::create_simple_mock(10000.0, 10000.0),

            compartments: indexmap::IndexMap::new(),
            hold_compartments: indexmap::IndexMap::new(),
            damaged_compartments: indexmap::IndexMap::new(),

            // Записываем наш мок парусности с готовыми значениями!
            windage_area: mock_windage,

            displacement_bounded: indexmap::IndexMap::new(),
            compartments_bounded: indexmap::IndexMap::new(),
            hold_compartments_bounded: indexmap::IndexMap::new(),

            // Создаем безопасный пул потоков
            thread_pool: Arc::new(sal_sync::thread_pool::ThreadPool::new(
                "ModelCached::mock_with_strength_areas",
                None,
            )),
        }
    }
}
