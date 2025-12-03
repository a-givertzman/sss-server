mod build_cache;
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
}