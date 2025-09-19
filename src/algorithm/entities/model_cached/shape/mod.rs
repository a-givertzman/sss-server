//!
//! Defines a shape trait
mod area_shape;
mod displacement_shape;
mod file_io;

pub(crate) use area_shape::*;
pub(crate) use displacement_shape::*;
pub(crate) use file_io::*;

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
        let center = self
            .center()
            .ok_or(Error::new(self.dbg(), "position").err("no center"))?;
        Ok(position(center, heel, trim, draught))
    }

    /// Разбиение меша по высоте на draught_qnt_steps шагов.
    /// Макимальный и минимальный уровень считаются с учетом наклона
    fn draught_steps(&self, draught_qnt_steps: usize) -> Result<Vec<f64>, Error> {
        let error = Error::new(self.dbg(), "draught_steps");
        let mesh = self.mesh().ok_or(error.err("no mesh"))?;
        let aabb = mesh.local_aabb();
        let center = if let Some(center) = self.center() {
            center
        } else {
            &compartment_center(mesh)
        };
        assert!(aabb.maxs.x >= center.x);
        assert!(aabb.mins.x <= center.x);
        assert!(aabb.maxs.y >= center.y);
        assert!(aabb.mins.y <= center.y);    
        let max_dx = (aabb.maxs.x - center.x).max(center.x - aabb.mins.x);
        let max_dy = (aabb.maxs.y - center.y).max(center.y - aabb.mins.y);
        let max_dz = max_dy.max(max_dx);
        if draught_qnt_steps <= 1 {
            return Err(error.err("draught_qnt_steps <= 1"));
        }
        let mut result = vec![];
        let min_z = aabb.mins.z - max_dz;
        let max_z = aabb.maxs.z + max_dz;
        let mut current = min_z;
        let step = (max_z - min_z)/(draught_qnt_steps as f64 - 1.);
        while current < max_z {
            result.push(current);
            current += step;
        }
        result.push(max_z);
        Ok(result)
    }
  /*  fn draught_steps(&self, draught_qnt_steps: usize) -> Result<Vec<f64>, Error> {
        let error = Error::new(self.dbg(), "draught_steps");
        let aabb = self.mesh().ok_or(error.err("no mesh"))?.local_aabb();
        if draught_qnt_steps <= 1 {
            return Err(error.err("draught_qnt_steps <= 1"));
        }
        if aabb.mins.z >= aabb.maxs.z {
            return Err(error.err("aabb.mins.z >= aabb.maxs.z"));
        }
        let mut result = vec![];
        let mut current = aabb.mins.z;
        let step = (aabb.maxs.z - aabb.mins.z)/(draught_qnt_steps as f64 - 1.);
        while current < aabb.maxs.z {
            result.push(current);
            current += step;
        }
        result.push(aabb.maxs.z);
        Ok(result)
    }*/
}
///
/// Расчет начала координат для отсеков как
/// проекции центра объема модели на ее нижнюю плоскость
pub(crate) fn compartment_center(mesh: &TriMesh) -> Point3<f64> {
    let properties = parry3d_f64::shape::Shape::mass_properties(mesh, 1.);
    let aabb: Aabb = mesh.local_aabb();
    Point3::new(properties.local_com.x, properties.local_com.y, aabb.mins.z)
}

/// Расчет положения корпуса
pub fn position(center: &Point3<f64>, heel: f64, trim: f64, draught: f64) -> Isometry3<f64> {
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
    Isometry::from_parts(translation, rotation)
}
