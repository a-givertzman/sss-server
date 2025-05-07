//!
//! The representation of the ship in terms of its 3D elements.
//
mod floating_position;
pub mod local_cache;
mod model_tree;
pub mod relative_position;
pub mod ship_model_conf;
use floating_position::FloatingPosition;
//
use indexmap::{IndexMap, IndexSet};
use local_cache::{
    cache_key::CacheKey, floating_position_cache::FloatingPositionCache, LocalCache,
};
use model_tree::ModelTree;
use relative_position::RelativePostion;
use sal_3dlib::{
    props::Center,
    topology::shape::{
        compound::{AlgoMakerVolume, Compound},
        edge::Edge,
        face::Face,
        vertex::Vertex,
        Shape,
    },
};
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
use ship_model_conf::ShipModelConf;
use std::sync::Arc;
///
/// Ship object represented as a collection of its 3D elements all with attributes of type `A`.
///
/// See [sal_3dlib::props::Attributes] to get more details about what the attribute type is.
pub struct ShipModel<A> {
    dbgid: DbgId,
    ///
    /// Privides access to structure of the 3D element by keys.
    model_tree: ModelTree<A>,
    ///
    /// Provides a number of calculations:
    /// - Floating position (see [FloatingPositionCache]).
    caches: IndexMap<CacheKey, Box<dyn LocalCache>>,
}
//
//
impl<A: Clone + Send + 'static> ShipModel<A> {
    ///
    /// Creates a new instance.
    pub fn new(parent: &DbgId, conf: ShipModelConf) -> Self {
        let dbgid = DbgId::with_parent(parent, "ShipModel");
        let model_tree = ModelTree::new(&dbgid, conf.model_path);
        let mut ship_model = Self {
            caches: IndexMap::new(),
            model_tree: model_tree.clone(),
            dbgid: dbgid.clone(),
        };
        ship_model.caches.insert(
            CacheKey::FloatingPostion,
            Box::new(FloatingPositionCache::new(
                &dbgid,
                model_tree,
                conf.cache_dir,
                conf.floating_position_cache_conf,
            )),
        );
        ship_model
    }
    ///
    /// Returns model elements touched by `waterline` and filtered by [RelativePostion].
    ///
    /// The algorithm uses those elements of the `self.model_tree`, which are specified in `keys`.
    /// If `keys` is empty it's considered to use all model elements.
    /// _Note_ that in the both cases only those elements are used, which types can make volume.
    /// In particular, these types are [Face]s, [Shell]s, and [Solid]s.
    ///
    /// # Examples
    /// ```
    /// use sal_3dlib::topology::shape::Face;
    /// use sal_sync::services::entity::error::str_err::StrErr;
    /// //
    /// // waterline constructor that creates a face (kind of plane)
    /// // based on x, y, and z coordinates - the waterline center
    /// fn create_waterline<T>(x: f64, y: f64, z: f64) -> Face<T> {
    ///     /* ... */
    /// }
    /// //
    /// //
    /// fn algorithm<T>(ship_model: &ShipModel<T>) -> Result<(), StrErr> {
    ///     let waterline = create_waterline(0.0, 0.0, 0.0);
    ///     // split an element of the target ship model (consider there is one called 'hull')
    ///     // and filter result elements to get those, which are above created waterline plane
    ///     let _ = ship_model.subvolume(&["hull"], &waterline, RelativePostion::Above)?;
    /// }
    /// ```
    ///
    /// [Shell]: sal_3dlib::topology::shape::Shell
    /// [Solid]: sal_3dlib::topology::shape::Solid
    pub fn subvolume(
        &self,
        keys: &[&str],
        waterline: &Face<Option<A>>,
        relative_position: RelativePostion,
    ) -> Result<Vec<Shape<Option<A>>>, StrErr> {
        let dbgid = DbgId(format!("{}.subvolume", self.dbgid));
        // pop up warning if a key is not present in `self.model_key`
        for key in keys {
            if !self.model_tree.contains_key(key) {
                log::warn!("{} | No element found for key='{}'", dbgid, key);
            }
        }
        // defines whether the key should be taken
        let should_volume = |key| keys.is_empty() || self.model_tree.contains_key(key);
        let [.., waterline_z] = waterline.center().point();
        self.model_tree
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
                let elmnt = build?;
                let [.., elmnt_z] = elmnt.center().point();
                if match relative_position {
                    RelativePostion::Above => elmnt_z > waterline_z,
                    RelativePostion::Under => elmnt_z < waterline_z,
                } {
                    elmnts.push(Shape::Compound(elmnt));
                }
                Ok(elmnts)
            })
    }
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
    pub fn update_caches(&mut self, caches: &[&CacheKey]) -> Result<(), StrErr> {
        // start wokers to calculate required caches
        let handlers = {
            let mut handlers = vec![];
            let calculate_all = caches.is_empty();
            for (cache_key, cache) in &self.caches {
                if calculate_all || caches.contains(&cache_key) {
                    cache
                        .calculate(Arc::default())
                        .map(|workers| handlers.push((*cache_key, workers)))?;
                }
            }
            handlers
        };
        // Get keys of successfuly calculated caches.
        // Return the full error if any worker fails.
        let calculated = {
            let dbgid = DbgId(format!("{}.update_caches", self.dbgid));
            let mut calculated = IndexSet::new();
            let mut errors = vec![];
            for (cache_key, workers) in handlers {
                for (id, handler) in workers {
                    match handler.join() {
                        Err(err) => {
                            log::error!("{} | Preparing thread='{}'..", dbgid, id);
                            errors.push(format!("  thread_id='{}', {:?}", id, err));
                        }
                        Ok(worker_res) => {
                            if let Err(err) = worker_res {
                                log::error!("{} | Calculating cache in thread='{}'..", dbgid, id);
                                errors.push(format!("  thread_id='{}', {:?}", id, err));
                            } else {
                                calculated.insert(cache_key);
                            }
                        }
                    }
                }
            }
            if !errors.is_empty() {
                return Err(StrErr(errors.join("\n")));
            }
            calculated
        };
        for (cache_key, cache) in &mut self.caches {
            if calculated.contains(cache_key) {
                cache.reload();
            }
        }
        Ok(())
    }
    //
    //
    pub fn floating_position(
        &self,
        displacement: f64,
        displacement_center: [f64; 3],
        accuracy: f64,
    ) -> FloatingPosition<A> {
        //  TODO: set a correct value
        const WATER_DENCITY: f64 = 0.0;
        //
        FloatingPosition::new(
            &self.dbgid,
            self.caches
                .get(&CacheKey::FloatingPostion)
                .unwrap_or_else(|| {
                    panic!(
                        "{} | Trying to access uninitialized FloatingPositionCache",
                        self.dbgid
                    )
                })
                .as_ref(),
            self.centreline(),
            self.middle(),
            displacement,
            Vertex::new(displacement_center),
            WATER_DENCITY,
            accuracy,
        )
    }
    ///
    /// Returns the centreline.
    fn centreline(&self) -> Edge<A> {
        todo!("Return the centreline. Probably by building bounding box.")
    }
    ///
    /// Returns the middle plane.
    fn middle(&self) -> Face<A> {
        todo!("Return the middle plane. Probably by building bounding box.")
    }
}
