//!
//! Defines a shape trait 
mod file_io;
mod area_shape;
mod displacement_shape;

pub(crate) use file_io::*;
pub(crate) use area_shape::*;
pub(crate) use displacement_shape::*;

use nalgebra::*;
use parry3d_f64::bounding_volume::Aabb;
use parry3d_f64::shape::TriMesh;
use sal_core::dbg::Dbg;
use sal_core::error::Error;

pub trait Shape {
    //
    fn dbg(&self) -> &Dbg;
    //
    fn mesh(&self) -> Option<&TriMesh>; 
    //
    fn center(&self) -> Option<&Point3<f64>>; 
    /// Init shape, load geometry
    fn init(&mut self) -> Result<(), Error>; 
    ///
    /// Расчет положения корпуса
    fn position(&self, heel: f64, trim: f64, draught: f64) -> Result<Isometry3<f64>, Error> {
        let center  = self.center().ok_or(Error::new(self.dbg(), "position").err("no center"))?;
        let heel_rad = -heel.to_radians();
        let trim_rad = trim.to_radians();
        let trim_rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), trim_rad);
        let transformed_x_axis = trim_rotation.transform_vector(&Vector3::x_axis());
        let transformed_x_axis = UnitVector3::new_normalize(transformed_x_axis);
        let heel_rotation = UnitQuaternion::from_axis_angle(&transformed_x_axis, heel_rad);
        let rotation = heel_rotation * trim_rotation;
        let mut center = center.clone();
        center.z += draught;
        let point = rotation.transform_point(&center);
        let translation = Translation3::new(-point.x, -point.y, -point.z);
        Ok(Isometry::from_parts(translation, rotation))
    }
    /// Разбиение от draught_min до h_max меша на draught_step шаги
    fn draught_steps(
        &self,   
        draught_min: f64,
        draught_step: f64,
    ) -> Result<Vec<f64>, Error> {
        let error = Error::new(self.dbg(), "draught_steps");
        let aabb = self.mesh().ok_or(error.err("no mesh"))?.local_aabb();
        if draught_step <= 0. {
            return Err(error.err("draught_step == 0"))
        }
        if draught_min >= aabb.maxs.z {
            return Err(error.err("draught_min >= h_max"))
        }
        let mut result = vec![];
        let mut current = draught_min;
        while current < aabb.maxs.z {
            result.push(current);
            current += draught_step;
        } 
        result.push(aabb.maxs.z);
        Ok(result)
    }
}
///
/// Расчет начала координат для отсеков как
/// проекции центра объема модели на ее нижнюю плоскость
pub(crate) fn compartment_center(mesh: &TriMesh) -> Point3<f64> {
    let properties = parry3d_f64::shape::Shape::mass_properties(mesh, 1.);
    let aabb: Aabb = mesh.local_aabb();
    Point3::new(properties.local_com.x, properties.local_com.y, aabb.mins.z)
}
