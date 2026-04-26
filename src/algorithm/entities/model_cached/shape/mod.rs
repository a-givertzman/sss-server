//!
//! Defines a shape trait
mod area_shape;
mod displacement_shape;
mod utils;

pub(crate) use area_shape::*;
pub(crate) use displacement_shape::*;
pub(crate) use utils::*;

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
    ///
    /// Расчет положения корпуса
    fn position_yz(&self, heel: f64, trim: f64, draught: f64) -> Result<Isometry3<f64>, Error> {
        let center = self
            .center()
            .ok_or(Error::new(self.dbg(), "position").err("no center"))?;
        Ok(position_yz(center, heel, trim, draught))
    }
    /// Разбиение меша по высоте на draught_qnt_steps шагов.
    /// Макимальный и минимальный уровень считаются с учетом наклона
    fn draught_by_step(&self, step: f64, max_heel: f64, max_trim: f64) -> Result<Vec<f64>, Error> {
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
        let mut result = vec![]; 
        let max_dx = (aabb.maxs.x - center.x).max(center.x - aabb.mins.x);
        let max_dy = (aabb.maxs.y - center.y).max(center.y - aabb.mins.y);
        let max_dz = max_dx*max_trim.to_radians().tan().abs() + max_dy*max_heel.to_radians().tan().abs()*max_trim.to_radians().cos();
        let delta_z = aabb.maxs.z - aabb.mins.z;
        let min_z = -max_dz;
        let max_z = delta_z + max_dz;
        let mut current = min_z;
        // Идем от нижней точки к верхней с заданным шагом
        // Используем небольшой эпсилон, чтобы не пропустить max_z из-за точности f64
        while current < max_z - f64::EPSILON {
            result.push(current);
            current += step;
        }
        // Всегда добавляем самую верхнюю точку в конце
        result.push(max_z);
        Ok(result)
    }
    /// Разбиение меша по высоте на draught_qnt_steps шагов.
    /// Макимальный и минимальный уровень считаются с учетом наклона
    fn draught_steps(&self, level_step_qnt: usize, max_heel: f64, max_trim: f64) -> Result<Vec<f64>, Error> {
        assert!(level_step_qnt > 2);
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
        let mut result = vec![]; 
        let max_dx = (aabb.maxs.x - center.x).max(center.x - aabb.mins.x);
        let max_dy = (aabb.maxs.y - center.y).max(center.y - aabb.mins.y);
        let max_dz = max_dx*max_trim.to_radians().tan().abs() + max_dy*max_heel.to_radians().tan().abs()*max_trim.to_radians().cos();
        let delta_z = aabb.maxs.z - aabb.mins.z;
        let min_z = -max_dz;
        let max_z = delta_z + max_dz;
        let step = delta_z/(level_step_qnt as f64 - 1.);    
        let step_max_dz = if max_dz*2. > delta_z {
            2.*max_dz/(level_step_qnt as f64 - 1.)
        } else {
            step
        };  
        let mut current = min_z;
        let mut current_step = step_max_dz;
        while current + current_step/2. <= 0. {
            result.push(current);
            current += current_step;
        }
        current = 0.;
        current_step = step;
        while current + current_step/2. <= delta_z {
            result.push(current);
            current += current_step;
        }
        current = delta_z;
        current_step = step_max_dz;
        while current + current_step/2. <= max_z {
            result.push(current);
            current += current_step;
        }
        result.push(max_z);
      //  log::debug!("shape draught_steps max_dx:{} max_dy:{} max_dz:{} delta_z:{} step_max_dz:{} min_z:{} max_z:{} aabb.mins.z:{} aabb.maxs.z:{} level_step_qnt:{}", 
      //      max_dx, max_dy, max_dz, delta_z, step_max_dz, min_z, max_z, aabb.mins.z, aabb.maxs.z, level_step_qnt);
      //  log::debug!("shape draught_steps {:?}", result);
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
///
/// Расчет положения корпуса
pub fn position(center: &Point3<f64>, heel: f64, trim: f64, draught: f64) -> Isometry3<f64> {
    let heel_rad = heel.to_radians();
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
///
/// Расчет положения корпуса без смещения по X
pub fn position_yz(center: &Point3<f64>, heel: f64, trim: f64, draught: f64) -> Isometry3<f64> {
    let heel_rad = heel.to_radians();
    let trim_rad = trim.to_radians();
    let trim_rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), trim_rad);
    let transformed_x_axis = trim_rotation.transform_vector(&Vector3::x_axis());
    let transformed_x_axis = UnitVector3::new_normalize(transformed_x_axis);
    let heel_rotation = UnitQuaternion::from_axis_angle(&transformed_x_axis, heel_rad);
    let rotation = heel_rotation * trim_rotation;
    let mut center = center.clone();
    center.z += draught;
    let point = rotation.transform_point(&center);
    let translation = Translation3::new(0.0, 0.0, 0.0);
    Isometry::from_parts(translation, rotation)
}