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
use sal_core::{dbg::Dbg, error::Error};
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
use ship_model_conf::ShipModelConf;
use std::sync::Arc;
///
/// Ship object represented as a collection of its 3D elements all with attributes of type `A`.
///
/// See [sal_3dlib::props::Attributes] to get more details about what the attribute type is.
pub struct ShipModel<A> {
    dbg: Dbg,
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
    pub fn new(parent: &Dbg, conf: ShipModelConf) -> Self {
        let dbg = Dbg::new(parent, "ShipModel");
        let model_tree = ModelTree::new(&dbg, conf.model_path);
        let mut ship_model = Self {
            caches: IndexMap::new(),
            model_tree: model_tree.clone(),
            dbg: dbg.clone(),
        };
        ship_model.caches.insert(
            CacheKey::FloatingPostion,
            Box::new(FloatingPositionCache::new(
                &dbg,
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
    pub fn subvolume(
        &self,
        keys: &[&str],
        waterline: &Face<Option<A>>,
        relative_position: RelativePostion,
    ) -> Result<Vec<Shape<Option<A>>>, Error> {
        let error = Error::new(&self.dbg, "subvolume");
        // pop up warning if a key is not present in `self.model_key`
        for key in keys {
            if !self.model_tree.contains_key(key) {
                log::warn!("{} subvolume | No element found for key='{}'", self.dbg, key);
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
                let elmnt = build.map_err(|err| error.pass_with("self.model_tree error", err.to_string()))?;
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
    pub fn update_caches(&mut self, caches: &[&CacheKey]) -> Result<(), Error> {
        // start wokers to calculate required caches
        let errors = {
            let mut errors = vec![];
            let calculate_all = caches.is_empty();
            for (cache_key, cache) in &self.caches {
                if calculate_all || caches.contains(&cache_key) {
                    errors.push((*cache_key, cache.calculate(Arc::default())));
                }
            }
            errors
        };
        // Get keys of successfuly calculated caches.
        // Return the full error if any worker fails.
        let calculated = {
            let error = Error::new(&self.dbg, "update_caches");
            let mut calculated = IndexSet::new();
            let mut filtered_errors = vec![];
            for (cache_key, errors) in errors {
                if errors.is_empty() {
                    calculated.insert(cache_key);
                } else {
                    for err in &errors {
                        let err_text = format!(" {} | Calculating cache: {:?} in thread error: {:?}", &self.dbg, cache_key, err);
                        log::error!("{}", &err_text);
                        filtered_errors.push(err_text);  
                    }            
                }
            }
            if !filtered_errors.is_empty() {
                return Err(error.pass_with("calculated", filtered_errors.join("\n")));
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
        //
        FloatingPosition::new(
            &self.dbg,
            self.caches
                .get(&CacheKey::FloatingPostion)
                .unwrap_or_else(|| {
                    panic!(
                        "{} | Trying to access uninitialized FloatingPositionCache",
                        self.dbg
                    )
                })
                .as_ref(),
            self.centreline(),
            self.middle(),
            displacement,
            Vertex::new(displacement_center),
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
