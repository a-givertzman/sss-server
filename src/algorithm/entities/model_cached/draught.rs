use nalgebra::Point3;
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
        let theta_rad = self.heel.min(89.9999999).max(-89.9999999).to_radians();
        let phi_rad = self.trim.min(89.9999999).max(-89.9999999).to_radians();
        let tg_theta = theta_rad.tan();
        let cos_theta = theta_rad.cos();        
        let tg_phi = phi_rad.tan();
        let draught = |point: Point3<f64>| {
            dbg!(point, tg_theta, cos_theta, tg_phi);
            self.draught_mid + point.y * tg_theta + (point.x - self.midel_x) * tg_phi / cos_theta
        };
        let bow = Point3::new(self.length_lbp, 0.0, -self.draught_mid);
        let stern = Point3::new(0., 0.0, -self.draught_mid);
        let mean = Point3::new(self.waterline_x, self.waterline_y, -self.draught_mid);
        (draught(bow), draught(stern), draught(mean))
    }
    /*
    /// Расчет осадок
    /// (draught_bow, draught_stern, draught_mean)
    pub fn calculate(&self) -> (f64, f64, f64) {
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
        let bow = Point3::new(self.length_lbp / 2., 0.0, -self.draught_mid);
        let stern = Point3::new(-self.length_lbp / 2., 0.0, -self.draught_mid);
        let mean = Point3::new(
            self.waterline_x - self.length_lbp / 2.,
            self.waterline_y,
            -self.draught_mid,
        );
        let draught = |point: Point3<f64>| {
            let dir = rotation.transform_vector(&Vector3::z_axis());
            let origin = rotation.transform_point(&point);
            let ray = Ray::new(origin, dir);
            let result = water_plane.cast_local_ray(&ray, self.length_lbp, true);
            /*     println!("p:({:.6} {:.6} {:.6}) dir:({:.6} {:.6} {:.6}) origin:({:.6} {:.6} {:.6}) r:{:.6}",
                point.x, point.y, point.z,
                dir.x, dir.y, dir.z,
                origin.x, origin.y, origin.z,
                result.unwrap_or(0.)
            );*/
            match result {
                Some(v) => v,
                None => 0.,
            }
        };
        (draught(bow), draught(stern), draught(mean))
    }
    */
}
