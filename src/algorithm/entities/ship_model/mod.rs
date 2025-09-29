//pub mod query;
//pub mod reply;
pub mod ship_model;
mod grain_moment;

use crate::algorithm::entities::{Bounds, Moment, Position};

///
/// Структура для ввода данных расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceQuery {
    /// Плотность забортной воды
    pub water_density: f64,
    /// масса судна порожнем и грузов размещенных на судне: 
    /// генерального груза (unitCargoAssignment), контейнеров (containerCargoAssignment), 
    /// газообразного груза (gaseousCargoAssignment), массы обледенения и намокания;
    pub mass_const: f64,
    /// Сумарный момент за вычетом смещяемых и насыпных грузов
    pub moment_const: Moment,
    /// навалочный груз
    pub bulk: Vec<BulkData>,
    /// жидкий груз
    pub liquid: Vec<LiquidData>,
    /// газообразный груз
    pub gaseous: Vec<GaseousData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>,
//    /// номера поврежденных помещений, TODO - только для аварийного расчета
//    pub damaged_compartment: Vec<String>,
    /// точность расчета
    pub epsilon: f64,
    /// шпации разбиения
    pub bounds: Bounds,
}
///
/// Груз, для которого центр массы и распределение зависит от 
/// объема и/или положения корпуса, считается в модели
/// Сыпучий груз
#[derive(Debug, Clone)]
pub struct BulkData {
    pub cargo_id: usize, // ID груза
    pub space_id: String, // ID помещения
    pub mass: f64,
    pub volume: f64,
}
///
/// Жидкий груз
#[derive(Debug, Clone)]
pub struct LiquidData {
    pub cargo_id: usize, // ID груза
    pub space_id: String, // ID помещения
    pub mass: f64,
    pub volume: f64,
}
///
/// Газообразный груз
/// Для него распределение не зависит от обьема или массы,
/// считаем распределение по отсеку при максимальном объеме отсека
#[derive(Debug, Clone)]
pub struct GaseousData {
    pub cargo_id: usize, // ID груза
    pub space_id: String, // ID помещения
    pub mass: f64,
}
///
#[derive(Debug, Clone)]
pub struct BoundArea {
    pub v: Vec<f64>,
    pub h: Vec<f64>,
}
///
/// Структура для данных результата расчета баланса судна.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceResult {
    /// Крен, градусы
    pub heel: f64,
    /// Дифферент, градусы
    pub trim: f64,
    /// осадка на миделе
    pub draught_mid: f64, 
    /// осадка на носовом перпендикуляре
    pub draught_bow: f64, 
    /// осадка на кормовом перпендикуляре
    pub draught_stern: f64,
    /// средняя осадка (в центре тяжести ватерлинии)
    pub draught_mean: f64, 
    /// Объемное водоизмещение, м^3
    pub displacement: f64,
    /// Смещение центра объемного водоизмещения, м
    pub displacement_center: Position,
    /// распределение водоизмещения по шпациям
    pub displacement_distr: Vec<f64>,  


    /// распределение массы газообразных грузов по шпациям 
    pub gaseous_distr: Vec<f64>,
    /// распределение массы сыпучих грузов по шпациям 
    pub bulk_distr: Vec<f64>,    
    /// распределение массы жидких грузов по шпациям 
    pub liquid_distr: Vec<f64>,  


    /// Площадь ватерлинии, м^2
    pub area_wl: f64,
    /// Смещение центра тяжести ватеринии, м
    pub area_wl_center: Position,
    /// Длинна по ватерлинии при текущей осадке, м
    pub length_wl: f64, 
    ///  Ширина по ватерлинии при текущей осадке, м
    pub breadth_wl: f64, 
    /// Продольный метацентрический радиус, м
    pub rad_long: f64,
    /// Поперечный метацентрические радиус, м
    pub rad_trans: f64,
   // ///  Угол входа в воду кромки палубы, градусы
  //  pub entry_angle: f64, 
  //  ///  Угол заливания отверстий, градусы
  //  pub flooding_angle: f64, 
  //  /// Суммарная площадь проекции на диаметральную плоскость, в пределах  
  //  /// 0,15 LBP в корму от носового перпендикуляра, части корпуса судна  
  //  /// между ватерлинией и линией палубы у борта и закрытой надстройки, м^2
  //  pub bow_area: f64,
  //  /// Площади боковой и горизонтальной поверхностей для расчета остойчивости, м^2
  //  pub const_area_v: Vec<(f64, Position)>,
  //  pub const_area_h: Vec<(f64, Position)>,
  //  /// Массив значений плечей от крена для текущих значений дифферента и осадки, м/градусы 
  //  pub pantocaren: Vec<(f64, f64)>,    
}






