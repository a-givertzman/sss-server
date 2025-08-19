use std::path::PathBuf;

use crate::{
    algorithm::entities::model_cached::{
        AreaCache, BoundedAreaCache, CompartmentCache, DisplacementCache, Shape,
        floating_position::{EvaluatedFloatingPosition, FloatingPosition},
        load_stl,
    },
    kernel::types::{Arc, RwLock},
};
use indexmap::{IndexMap, IndexSet};
use sal_core::{dbg::Dbg, error::Error};
/*use sal_3dlib::{
    props::Center,
    topology::shape::{
        compound::{AlgoMakerVolume, Compound},
        edge::Edge,
        face::Face,
        vertex::Vertex,
        Shape,
    },
};*/
use crate::algorithm::entities::Position2d;
use sal_sync::{
    sync::Stack,
    thread_pool::{JoinHandle, Scheduler},
};

//use super::floating_position::FloatingPosition;
use super::{LocalCache, ModelCachedConf};

//
/*  pub struct EvaluatedFloatingPosition {
    pub heel_angle: f64,
    pub trim_angle: f64,
    pub draught_at_amidships: f64,
    pub displacement: f64,
    pub disp_center: Position,
    bulk: Vec<BulkResult>,
    liquid: Vec<LiquidResult>,
    damage_compartment: Vec<CompartmentResult>,
}*/
///
/// Ship object represented as a collection of its 3D elements all with attributes of type `A`.
///
/// See [sal_3dlib::props::Attributes] to get more details about what the attribute type is.
pub struct ModelCached {
    dbg: Dbg,
    ///
    /// Privides access to structure of the 3D element
    shapes: Vec<Arc<RwLock<Shape>>>,
    /// Provides a number of calculations:
    /// - cache for model, [heel, trim, draught, volume, x, y, z, area, x, y, z, l_x, l_y, i_x, i_y ]
    displacement: DisplacementCache,
    /// - cache for compartments, [index of compartments, [heel, trim, level, volume, x, y, z, i_x, i_y ]]
    compartments: IndexMap<String, CompartmentCache>,
    /// - cache for bounds of model, [index of bound, [trim, draught, volume ]]
    //   model_bounded: IndexMap<usize, Vec<BoundCache>>,
    /// - cache for bounds of compartments,  [index of bound, TODO]
    //    compartments_bounded: IndexMap<usize, IndexMap<usize, IndexMap<usize, BoundCache>>>,
    /// - cache for windage area
    windage_area: AreaCache,
    /// - cache for bounded windage area
    bounded_windage_area: BoundedAreaCache,
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
        let mut shapes = Vec::new();
        let delta_pos = Some(conf.model_center_coord.clone());
        let displacement_shape = Arc::new(RwLock::new(Shape::new_uninit(
            &dbg,
            conf.model_dir.clone().join(PathBuf::from("hull.stl")),
            None,
            delta_pos,
            conf.model_scale,
        )));
        shapes.push(displacement_shape.clone());
        let windage_shape = Arc::new(RwLock::new(Shape::new_uninit(
            &dbg,
            conf.model_dir.clone().join(PathBuf::from("hull.stl")),
            Some(conf.model_dir.clone().join(PathBuf::from("additionals"))),
            delta_pos,
            conf.model_scale,
        )));
        shapes.push(windage_shape.clone());
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
            .into_iter()
            .filter(|path| path.file_name().is_some())
            .map(|path| {
                let Some(name) = path.file_stem() else {
                    return None;
                };
                let Some(name) = name.to_str() else {
                    return None;
                };
                let name = name.to_string();
                let shape = Arc::new(RwLock::new(Shape::new_uninit(
                    &dbg,
                    path,
                    None,
                    None,
                    conf.model_scale,
                )));
                shapes.push(shape.clone());
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
        let model_cached = Self {
            dbg: dbg.clone(),
            shapes,
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
            windage_area: AreaCache::new(
                &dbg,
                windage_shape.clone(),
                conf.cache_dir.clone(),
                conf.trim_steps.clone(),
                conf.draught_min,
                conf.hull_draught_step,
                scheduler.clone(),
            ),
            bounded_windage_area: BoundedAreaCache::new(
                &dbg,
                windage_shape.clone(),
                conf.cache_dir.clone(),
                conf.trim_steps.clone(),
                conf.draught_min,
                conf.hull_draught_step,
                scheduler.clone(),
            ),
            compartments,
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
        for shape in &self.shapes {
            let shape = shape.clone();
            let task_results = task_results.clone();
            let handle = self
                .scheduler
                .spawn(move || {
                    let mut guard = shape.write();
                    task_results.push(guard.init());
                    Ok(())
                })
                .map_err(|err| error.pass_with(format!("spawn task shape"), err.to_string()));
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
            errors.push(("windage_area".to_owned(), error));
        }
        dbg!("windage_area end");
        dbg!("bounded_windage_area start");
        if let Err(error) = self.bounded_windage_area.rebuild() {
            errors.push(("bounded_windage_area".to_owned(), error));
        }
        dbg!("bounded_windage_area end");
        dbg!("compartments start");
        for (name, compartment) in &mut self.compartments {
            if let Err(error) = compartment.rebuild() {
                errors.push((("compartment ".to_owned() + name), error));
            }
        }
        dbg!("compartments end");
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
    //
    pub fn floating_position(
        &self,
        displacement: f64,
        mass_center: Position2d,
        //    bulk: Vec<BulkLoad>,
        //    liquid: Vec<LiquidLoad>,
        //    damage_compartment: Vec<usize>,
    ) -> Result<EvaluatedFloatingPosition, Error> {
        //
        FloatingPosition::new(&self.dbg, &self.displacement, displacement, mass_center).eval()
    }
    /*
    pub fn floating_position(
        &self,
        displacement: f64,
        mass_center: Position2d,
    ) -> FloatingPosition {
        //
        FloatingPosition::new(
            &self.dbg,
            self.caches
                .get(&CacheKey::FloatingPostion)
                .unwrap_or_else(|| {
                    panic!(
                        "{} | Trying to access uninitialized DisplacementCache",
                        self.dbg
                    )
                })
                .as_ref(),
     //       self.centreline(),
     //       self.middle(),
            displacement,
            mass_center,
        )
    }
    ///
    /// Returns the centreline.
    fn centreline(&self) -> Edge<ModelCachedMeta> {
        todo!("Return the centreline. Probably by building bounding box.")
    }
    ///
    /// Returns the middle plane.
    fn middle(&self) -> Face<ModelCachedMeta> {
        todo!("Return the middle plane. Probably by building bounding box.")
    }
    */
}
