use nalgebra::{Isometry, Point3, UnitQuaternion, UnitVector3, Vector3};
use parry3d_f64::{math::{Translation, UnitVector}, query::{Ray, RayCast}};
use parry3d_f64::shape::HalfSpace;

use crate::algorithm::entities::model_cached::position;
///
/// Осадкт судна. Считаются из осадки на миделе и параметров судна
pub struct Draught {
    midel_x: f64,             
    length_lbp: f64,
    draught_mid: f64, 
    waterline_x: f64, 
    waterline_y: f64,    
    heel: f64,    
    trim: f64,
}
//
//
impl Draught {
    ///
    /// Главный конструктор
    /// * midel_x - смещение миделя по Х
    /// * length_lbp - длинна корпуса судна между перпендикулярами
    /// * draught_mid - осадка на миделе 
    /// * waterline_x - смещение центра тяжести ватеринии по Х
    /// * waterline_y - смещение центра тяжести ватеринии по Y
    /// * heel - крен в градусах
    /// * trim - дифферент в градусах
    pub fn new(
        midel_x: f64,         
        length_lbp: f64,
        draught_mid: f64,
        waterline_x: f64,
        waterline_y: f64,
        heel: f64,
        trim: f64,
    ) -> Self {
        Self {
            midel_x,            
            length_lbp,
            draught_mid,
            waterline_x,
            waterline_y,   
            heel,   
            trim,
        }
    }

   /// Расчет осадок
    /// (draught_bow, draught_stern, draught_mean)
    pub fn calculate(&self) -> (f64, f64, f64) {
    //    let center = Point3::new(0., 0., self.draught_mid);
    // расчет ориентации корпуса
        let rotation = {
            let heel_rad = -self.heel.to_radians();
            let trim_rad = self.trim.to_radians();
            let trim_rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), trim_rad);
            let transformed_x_axis = trim_rotation.transform_vector(&Vector3::x_axis());
            let transformed_x_axis = UnitVector3::new_normalize(transformed_x_axis);
            let heel_rotation = UnitQuaternion::from_axis_angle(&transformed_x_axis, heel_rad);
            heel_rotation * trim_rotation
        };

        let water_plane = HalfSpace::new(-Vector3::z_axis());

        let bow = Point3::new(self.length_lbp/2., 0.0, -self.draught_mid);
        let stern = Point3::new(-self.length_lbp/2., 0.0, -self.draught_mid);
        let mean= Point3::new(self.waterline_x - self.length_lbp/2., self.waterline_y, -self.draught_mid);

        let draught = |point: Point3<f64>| {
            let dir = rotation.transform_vector(&Vector3::z_axis());
            let origin = rotation.transform_point(&point);
            let ray = Ray::new(origin, dir);
            let result = water_plane.cast_local_ray( &ray, self.length_lbp, true);
            println!("p:({:.6} {:.6} {:.6}) dir:({:.6} {:.6} {:.6}) origin:({:.6} {:.6} {:.6}) r:{:.6}", 
                point.x, point.y, point.z, 
                dir.x, dir.y, dir.z, 
                origin.x, origin.y, origin.z, 
                result.unwrap_or(0.)
            );
            match result {
                Some(v) => v,
                None => 0.,
            }
        };

        let draught_bow = draught(bow);
        let draught_stern = draught(stern);
        let draught_mean = draught(mean);        
     //       dbg!(bow_x, stern_x, trim_m, draught_bow, draught_stern, draught_mean);
        (draught_bow, draught_stern, draught_mean)
    }

/*    
    /// Расчет осадок
    /// (draught_bow, draught_stern, draught_mean)
    pub fn calculate(&self) -> (f64, f64, f64) {
    // расчет ориентации корпуса
        let transform = {
            let heel_rad = -self.heel.to_radians();
            let trim_rad = self.trim.to_radians();
            let trim_rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), trim_rad);
            let transformed_x_axis = trim_rotation.transform_vector(&Vector3::x_axis());
            let transformed_x_axis = UnitVector3::new_normalize(transformed_x_axis);
            let heel_rotation = UnitQuaternion::from_axis_angle(&transformed_x_axis, heel_rad);
            Isometry::from_parts(Translation::new(0., 0., -self.draught_mid), heel_rotation * trim_rotation)
        };

        let water_plane = HalfSpace::new(-Vector3::z_axis());

        let bow = Point3::new(self.length_lbp/2., 0.0, 0.0);
        let stern = Point3::new(-self.length_lbp/2., 0.0, 0.0);
        let mean= Point3::new(self.waterline_x - self.length_lbp/2., self.waterline_y, 0.0);

        let draught = |point: Point3<f64>| {
            let dir = transform.rotation.transform_vector(&Vector3::z_axis());
            let origin = transform.transform_point(&point);
            let ray = Ray::new(origin, dir);
            let result = water_plane.cast_local_ray_and_get_normal(&ray, self.length_lbp, true);
            println!("p:({:.6} {:.6} {:.6}) dir:({:.6} {:.6} {:.6}) origin:({:.6} {:.6} {:.6}) r:{:.6}", 
                point.x, point.y, point.z, 
                dir.x, dir.y, dir.z, 
                origin.x, origin.y, origin.z, 
                result.unwrap().time_of_impact
            );
            match result {
                Some(v) => v.time_of_impact,
                None => 0.,
            }
        };

        let draught_bow = draught(bow);
        let draught_stern = draught(stern);
        let draught_mean = draught(mean);        
     //       dbg!(bow_x, stern_x, trim_m, draught_bow, draught_stern, draught_mean);
        (draught_bow, draught_stern, draught_mean)
    }
*/

 /*   pub fn calculate(&self) -> (f64, f64, f64) {
        let bow_x = self.length_lbp - self.midel_x;
        let stern_x = -self.midel_x;
        let trim_m = self.trim.to_radians().sin()*self.length_lbp;
        let draught_bow = self.draught_mid + bow_x*trim_m/self.length_lbp;
        let draught_stern = self.draught_mid + stern_x*trim_m/self.length_lbp;
        let draught_mean = self.draught_mid + (self.waterline_x - self.midel_x)*trim_m/self.length_lbp 
            + self.waterline_y*self.trim.to_radians().cos()*self.heel.to_radians().sin();
     //       dbg!(bow_x, stern_x, trim_m, draught_bow, draught_stern, draught_mean);
        (draught_bow, draught_stern, draught_mean)
    }*/


/*
    /// Расчет осадок
    /// (draught_bow, draught_stern, draught_mean)
    pub fn calculate(&self) -> (f64, f64, f64) {
        let theta = self.heel.to_radians();
        let phi = self.trim.to_radians(); 
        let theta_sin= theta.sin();
        let theta_cos= theta.cos();
        let phi_sin= phi.sin();
        let phi_cos= phi.cos();
        let a1 = Matrix3::new(0., 0., 0., 
                                                                0., theta_cos, -theta_sin,
                                                                0., theta_sin, theta_cos);
        let a2 = Matrix3::new(phi_cos, 0., phi_sin, 
                                                                0., 1., 0.,
                                                                -phi_sin, 0., phi_cos);
        let a3 = Matrix3::new(phi_cos, -phi_sin, 0., 
                                                                phi_sin, phi_cos, 0.,
                                                                0., 0., 1.);
        let a = a1*a2*a3;
        let horizontal_plane = Vector3::new(0., 0., 1.);
        let waterline_plane = a*horizontal_plane;

        let plane_normal = Vector3::new(0.0, 1.0, 0.0);
        let plane = Plane::new(plane_normal);


        let d_i = d_middle + d_zi; 

        let bow_x = self.length_lbp - self.midel_x;
        let stern_x = -self.midel_x;
        let trim_m = self.trim.to_radians().sin()*self.length_lbp;
        let draught_bow = self.draught_mid + bow_x*trim_m/self.length_lbp;
        let draught_stern = self.draught_mid + stern_x*trim_m/self.length_lbp;
        let draught_mean = self.draught_mid + (self.waterline_x - self.midel_x)*trim_m/self.length_lbp 
            + self.waterline_y*self.trim.to_radians().cos()*self.heel.to_radians().sin();
     //       dbg!(bow_x, stern_x, trim_m, draught_bow, draught_stern, draught_mean);
        (draught_bow, draught_stern, draught_mean)
    }
*/
}
