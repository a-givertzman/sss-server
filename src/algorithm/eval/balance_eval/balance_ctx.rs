use bincode::{Decode, Encode};
use super::{bulk_result::BulkResult, liquid_result::LiquidResult};

///
#[derive(Debug, Clone, Decode, Encode)]
pub struct BalanceCtx {
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса.
    pub bulk: Vec<BulkResult>,
    pub liquid: Vec<LiquidResult>,
    /// Объемное водоизмещение
    pub volume: f64,
    /// Площадь ватерлинии
    pub area_wl: f64, 
    /// Средняя осадка
    pub mean_draught: f64,
    /// Длинна по ватерлинии при текущей осадке
    pub length_wl: f64, 
    ///  Ширина по ватерлинии при текущей осадке
    pub breadth_wl: f64, 
    ///  Отстояние по вертикали центра площади проекции подводной части корпуса
    pub volume_shift_z: f64, 
    ///  Угол входа в воду кромки палубы
    pub entry_angle: f64, 
    ///  Угол заливания отверстий
    pub flooding_angle: f64, 
}
