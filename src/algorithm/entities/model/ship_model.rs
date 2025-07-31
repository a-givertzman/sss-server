use crate::{
    algorithm::entities::model::{
        Shape,
        floating_position::{EvaluatedFloatingPosition, FloatingPosition},
    },
    model::DisplacementCache,
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
use sal_sync::thread_pool::Scheduler;

//use super::floating_position::FloatingPosition;
use super::{LocalCache, ShipModelConf};

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
pub struct ShipModel {
    dbg: Dbg,
    ///
    /// Privides access to structure of the 3D element by keys.
    //   model_shape: Shape,
    ///
    /// Provides a number of calculations:
    /// - cashe for model, [heel, trim, draught, volume, x, y, z, area, x, y, z, l_x, l_y, i_x, i_y ]
    model: DisplacementCache,
    //  model_bounded: Vec<BoundCache>,
    //   compartments: IndexMap<usize, CompartmentCache>,
    //   compartment_bounded: IndexMap<usize, IndexMap<usize, BoundCache>>,
    scheduler: Scheduler,
}
//
//
impl ShipModel {
    ///
    /// Creates a new instance.
    pub fn new(parent: &Dbg, conf: ShipModelConf, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(parent, "ShipModel");
        let ship_model = Self {
            dbg: dbg.clone(),
            model: DisplacementCache::new(
                &dbg,
                Shape::new_uninit(
                    &dbg,
                    conf.model_path,
                    conf.floating_position_cache_conf.center_coord.x(),
                    conf.model_scale,
                ),
                conf.cache_dir,
                conf.floating_position_cache_conf.heel_steps,
                conf.floating_position_cache_conf.trim_steps,
                conf.floating_position_cache_conf.draught_steps,
                scheduler.clone(),
            ),
            scheduler: scheduler.clone(),
        };
        ship_model
    }
    ///
    /// Returns model elements touched by `waterline` and filtered by [RelativePostion].
    ///
    /// The algorithm uses those elements of the `self.model_shape`, which are specified in `keys`.
    /// If `keys` is empty it's considered to use all model elements.
    /// _Note_ that in the both cases only those elements are used, which types can make volume.
    /// In particular, these types are [Face]s, [Shell]s, and [Solid]s.
    ///
    /// # Examples
    /// ```
    /// use sal_3dlib::topology::shape::Face;
    /// use sal_sync::services::entity::error::str_err::Error;
    /// //
    /// // waterline constructor that creates a face (kind of plane)
    /// // based on x, y, and z coordinates - the waterline center
    /// fn create_waterline<T>(x: f64, y: f64, z: f64) -> Face<T> {
    ///     /* ... */
    /// }
    /// //
    /// //
    /// fn algorithm<T>(ship_model: &ShipModel<T>) -> Result<(), Error> {
    ///     let waterline = create_waterline(0.0, 0.0, 0.0);
    ///     // split an element of the target ship model (consider there is one called 'hull')
    ///     // and filter result elements to get those, which are above created waterline plane
    ///     let _ = ship_model.subvolume(&["hull"], &waterline, RelativePostion::Above)?;
    /// }
    /// ```
    ///
    /// [Shell]: sal_3dlib::topology::shape::Shell
    /// [Solid]: sal_3dlib::topology::shape::Solid
    /*   pub fn subvolume(
        &self,
        keys: &[&str],
        waterline: &Face<ShipModelMeta>,
        relative_position: RelativePostion,
    ) -> Result<Vec<Shape<ShipModelMeta>>, Error> {
        let error = Error::new(&self.dbg, "subvolume");
        // pop up warning if a key is not present in `self.model_key`
        for key in keys {
            if !self.model_shape.contains_key(key) {
                log::warn!("{} subvolume | No element found for key='{}'", self.dbg, key);
            }
        }
        // defines whether the key should be taken
        let should_volume = |key| keys.is_empty() || self.model_shape.contains_key(key);
        let [.., waterline_z] = waterline.center().point();
        self.model_shape
            .iter()
            .filter_map(|(key, elmnt)| {
                Some(match elmnt {
                    Shape::Face(elmnt) if should_volume(key) => {
                        Compound::build([waterline, elmnt], [], [])
                    }
                    Shape::Shell(elmnt) if should_volume(key) => {
                        Compound::build([waterline], [elmnt], [])
                    }
                    Shape::Solid(elmnt) if should_volume(key) => {
                        Compound::build([waterline], [], [elmnt])
                    }
                    _ => return None,
                })
            })
            .try_fold(vec![], |mut elmnts, build| {
                let elmnt = build.map_err(|err| error.pass_with("self.model_shape error", err.to_string()))?;
                let [.., elmnt_z] = elmnt.center().point();
                if match relative_position {
                    RelativePostion::Above => elmnt_z > waterline_z,
                    RelativePostion::Under => elmnt_z < waterline_z,
                } {
                    elmnts.push(Shape::Compound(elmnt));
                }
                Ok(elmnts)
            })
    }*/
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
    ///
    /// # Examples
    /// ```
    /// fn explaination(ship_model: &mut ShipModel<()>) {
    ///     // reload all caches used by `ship_model`
    ///     if let Err(why) = ship_model.update_caches(&[]) {
    ///         println!("Failed to update ship model caches: {}", why);
    ///     }
    ///     // reload only the floating position cache
    ///     if let Err(why) = ship_model.update_caches(&[CacheKey::FloatingPostion]) {
    ///         println!("Failed to update the floating position cache: {}", why);
    ///     }
    /// }
    /// ```
    pub fn rebuild_caches(&mut self) -> Result<(), Error> {
        // start wokers to calculate required caches
        let mut errors = Vec::new();
        if let Err(error) = self.model.rebuild() {
            errors.push(("model", error));
        }
        let error = Error::new(&self.dbg, "rebuild_caches");
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
        FloatingPosition::new(&self.dbg, &self.model, displacement, mass_center).eval()
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
    fn centreline(&self) -> Edge<ShipModelMeta> {
        todo!("Return the centreline. Probably by building bounding box.")
    }
    ///
    /// Returns the middle plane.
    fn middle(&self) -> Face<ShipModelMeta> {
        todo!("Return the middle plane. Probably by building bounding box.")
    }
    */
}
