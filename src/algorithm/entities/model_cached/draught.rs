use nalgebra::{Point3, Vector3};
use parry3d_f64::{math::UnitVector, query::{Ray, RayCast}};
use parry3d_f64::shape::HalfSpace;

use crate::algorithm::entities::model_cached::position;
///
/// Pre-calculated cache for floating position algorithm.
/// contains [heel, trim, draught, volume, x, y, z, area, x, y, z, waterline_x, waterline_y]
pub struct Draught {
    heel: f64,
    trim: f64,
    draught: f64,
    center: Point3<f64>,
}
//
//
impl Draught {
    ///
    /// Creates a new instance.
    /// - cache_dir - folder contains all cache files
    pub fn new(
        heel: f64,
        trim: f64,
        draught: f64,
        center: Point3<f64>,
    ) -> Self {
        Self {
            heel,
            trim,
            draught,
            center,
        }
    }
    //
    fn calculate(&self, x: f64, y: f64) -> f64 {
        let isometry = position(
            &self.center,
            self.heel,
            self.trim,
            self.draught,  
        );
        let origin = isometry.transform_point(&Point3::new(x, y, 0.));
        let dir = UnitVector::new_normalize(isometry.rotation.transform_vector(&Vector3::new(0., 0., 1.)));      
        let plane = HalfSpace::new(UnitVector::new_normalize(Vector3::new(0., 0., -1.)));
        plane.cast_local_ray(
            &Ray::new(origin, *dir),           
            1000.,
            true,
        ).unwrap_or(1000.)
    }
}
