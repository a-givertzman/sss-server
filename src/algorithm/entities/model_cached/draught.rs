use nalgebra::{Point3, Vector3};
use parry3d_f64::{math::UnitVector, query::{Ray, RayCast}};
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
        let bow_x = self.length_lbp - self.midel_x;
        let stern_x = -self.midel_x;
        let trim_m = self.trim.to_radians().sin()*self.length_lbp;
        let draught_bow = self.draught_mid + bow_x*trim_m/self.length_lbp;
        let draught_stern = self.draught_mid + stern_x*trim_m/self.length_lbp;
        let draught_mean = self.draught_mid + (self.waterline_x - self.midel_x)*trim_m/self.length_lbp 
            + self.waterline_y*self.trim.to_radians().cos()*self.heel.to_radians().sin();
            dbg!(bow_x, stern_x, trim_m, draught_bow, draught_stern, draught_mean);
        (draught_bow, draught_stern, draught_mean)
    }
}
