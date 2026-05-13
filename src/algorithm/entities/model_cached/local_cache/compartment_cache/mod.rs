mod cache;

pub(crate) use cache::*;

use crate::algorithm::entities::Position;

/// Данные из кэша значений по отсекам
#[derive(Debug)]
pub struct CompartmentCacheResult {
    pub heel: f64,
    pub trim: f64,    
    pub level: f64,
    pub volume: f64,
    pub volume_center: Position, 
    /// Поперечный момент инерции площади ватерлинии относительно осей, параллельных осям X, м^4 
    pub inertia_trans_x: f64,    
    /// Продольный момент инерции площади ватерлинии относительно осей, параллельных осям Y, м^4 
    pub inertia_long_y: f64,  
    /// Максимальный поперечный момент инерции площади ватерлинии относительно осей, параллельных осям X, м^4 
    pub max_inertia_trans_x: f64,   
    /// Абсолютный момент жидкости при текущих углах и объеме
    pub abs_moment: f64,
//    /// Максимальный момент жидкости при текущем крене
 //   pub max_abs_moment: f64,
}
