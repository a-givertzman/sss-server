use parry3d_f64::math::Vec3;

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
        let theta_rad = self.heel.clamp(-89.9999999, 89.9999999).to_radians();
        let phi_rad = self.trim.clamp(-89.9999999, 89.9999999).to_radians();
        let tg_theta = theta_rad.tan();
        let cos_theta = theta_rad.cos();        
        let tg_phi = phi_rad.tan();
        let draught = |point: Vec3| {
            self.draught_mid + point.y * tg_theta + (point.x - self.midel_x) * tg_phi / cos_theta
        };
        let bow = Vec3::new(self.length_lbp, 0.0, -self.draught_mid);
        let stern = Vec3::new(0., 0.0, -self.draught_mid);
        let mean = Vec3::new(self.waterline_x, self.waterline_y, -self.draught_mid);
        (draught(bow), draught(stern), draught(mean))
    }
}
