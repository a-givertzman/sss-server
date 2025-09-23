mod build_cache;
//mod tests;
mod cache;

pub(crate) use cache::*;
use crate::algorithm::entities::Position;

/// Данные из кэша значений по корпусу
pub struct DisplacementCacheResult {
    pub heel: f64,
    pub trim: f64,    
    pub draught: f64,
    pub volume: f64,
    pub volume_center: Position, 
    pub area_wl: f64,    
    pub area_wl_center: Position, 
    /// Поперечный момент инерции площади ватерлинии относительно осей, параллельных осям X, м^4 
    pub inertia_trans_x: f64,    
    /// Продольный момент инерции площади ватерлинии относительно осей, параллельных осям Y, м^4 
    pub inertia_long_y: f64,       
    /// Длинна по ватерлинии при текущей осадке, м
    pub length_wl: f64,
    /// Ширина по ватерлинии при текущей осадке, м
    pub breadth_wl: f64,
}