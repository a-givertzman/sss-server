use bincode::{Decode, Encode};
use crate::algorithm::entities::{Position, ship_model::stability_result::{BulkResult, LiquidResult}};
//
#[derive(Debug, Clone, Decode, Encode)]
pub struct StabilityBalanceCtx {
    /// Крен, градусы
    pub heel: f64,
    /// Дифферент, градусы
    pub trim: f64,
    /// осадка на миделе
    pub draught_mid: f64,
    /// Смещение центра массы, м
    pub mass_center: Position,
    /// Объемное водоизмещение, м^3
    pub displacement: f64,
    /// Сыпучий груз для которого центр массы и распределение зависит от 
    /// объема
    pub bulk: Vec<BulkResult>,
    /// Жидкий груз, для которого центр массы и распределение зависит от 
    /// объема и положения корпуса
    pub liquid: Vec<LiquidResult>,
  //  /// Полное объемное водоизмещение, м^3
  //  pub volume: f64,
  //  /// Площадь ватерлинии, м^2
 //   pub area_wl: f64,
    /// Длинна по ватерлинии при текущей осадке, м
    pub length_wl: f64,
    ///  Ширина по ватерлинии при текущей осадке, м
    pub breadth_wl: f64,
    /// Суммарная площадь проекции на диаметральную плоскость, в пределах  
    /// 0,15 LBP в корму от носового перпендикуляра, части корпуса судна  
    /// между ватерлинией и линией палубы у борта и закрытой надстройки, м^2
    pub bow_area: f64,
 //   /// Площади боковой и горизонтальной поверхностей для расчета остойчивости, м^2
  //  pub const_area_v: Vec<(f64, Position)>,
 //   pub const_area_h: Vec<(f64, Position)>,
 //  /// Продольный метацентрический радиус, м
 //   pub rad_long: f64,
  //  /// Поперечный метацентрические радиус, м
  //  pub rad_trans: f64,
}
