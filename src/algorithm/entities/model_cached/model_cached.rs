use crate::algorithm::entities::model_cached::{
        floating_position::{EvaluatedFloatingPosition, FloatingPosition}, AreaCache, DisplacementCache, Shape};
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
    /// Privides access to structure of the 3D element by keys.
    //   model_shape: Shape,
    ///
    /// Provides a number of calculations:
    /// - cache for model, [heel, trim, draught, volume, x, y, z, area, x, y, z, l_x, l_y, i_x, i_y ]
    displacement: DisplacementCache,
    /// - cache for compartments, [index of compartments, [heel, trim, level, volume, x, y, z, i_x, i_y ]]
//    compartments: IndexMap<usize, CompartmentCache>,
    /// - cache for bounds of model, [index of bound, [trim, draught, volume ]]
 //   model_bounded: IndexMap<usize, Vec<BoundCache>>,    
    /// - cache for bounds of compartments,  [index of bound, TODO]
//    compartments_bounded: IndexMap<usize, IndexMap<usize, IndexMap<usize, BoundCache>>>,
    /// - cache for windage area
    vertical_area: AreaCache,
    scheduler: Scheduler,
}
//
//
impl ModelCached {
    ///
    /// Creates a new instance.
    pub fn new(parent: &Dbg, conf: ModelCachedConf, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(parent, "ModelCached");
        let model_cached = Self {
            dbg: dbg.clone(),
            displacement: DisplacementCache::new(
                &dbg,
                Shape::new_uninit(
                    &dbg,
                    conf.model_path,
                    conf.additional_path,
                    conf.cache_conf.center_coord.x(),
                    conf.model_scale,
                ),
                conf.cache_dir,
                conf.cache_conf.heel_steps,
                conf.cache_conf.trim_steps,
                conf.cache_conf.draught_steps,
                scheduler.clone(),
            ),
            vertical_area: todo!(),            
            scheduler: scheduler.clone(),
     //       compartments: todo!(),
    //        model_bounded: todo!(),
     //       compartments_bounded: todo!(),

        };
        model_cached
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
        // start wokers to calculate required caches
        let mut errors = Vec::new();
        if let Err(error) = self.displacement.rebuild() {
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
