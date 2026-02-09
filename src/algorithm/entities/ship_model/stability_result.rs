use bincode::{Decode, Encode};

use crate::algorithm::entities::{Position, data::loads::AssignmentType};
///
/// Структура для данных результата расчета баланса судна для остойчивости.
/// Содержит массу судна, грузов и положение зерновых перегородок
#[derive(Debug, Clone)]
pub struct BalanceStabilityResult {
    /// Крен, градусы
    pub heel: f64,
    /// Дифферент, градусы
    pub trim_degree: f64,
    /// Дифферент, метры
    pub trim_meter: f64,
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
    /// Смещение центра массы, м
    pub mass_center: Position,
   // /// Площадь ватерлинии, м^2
   // pub area_wl: f64,
  //  /// Смещение центра тяжести ватеринии, м
  //  pub area_wl_center: Position,
    /// Длинна по ватерлинии при текущей осадке, м
    pub length_wl: f64,
    /// Ширина по ватерлинии при текущей осадке, м
    pub breadth_wl: f64,
    /// Продольный метацентрический радиус, м
    pub rad_long: f64,
    /// Поперечный метацентрические радиус, м
    pub rad_trans: f64,
    /// Суммарная площадь проекции на диаметральную плоскость, в пределах
    /// 0,15 LBP в корму от носового перпендикуляра, части корпуса судна
    /// между ватерлинией и линией палубы у борта и закрытой надстройки, м^2
    pub bow_area: f64,
    //  /// Площади боковой и горизонтальной поверхностей для расчета остойчивости, м^2
    //  pub const_area_v: Vec<(f64, Position)>,
    //  pub const_area_h: Vec<(f64, Position)>,
    /// Данные сыпучих грузов
    pub bulk: Vec<BulkResult>,
    /// Данные жидких грузов
    pub liquid: Vec<LiquidResult>,
}
///
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct LiquidResult {
    /// ID assigned
    pub assignment_id: usize,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,  
    /// Масса груза
    pub mass: f64,
    /// Смещение центра массы груза
    pub mass_shift: Position, 
    /// продольный момент свободной поверхности жидкости
    pub long_moment_of_inertia: f64,
    /// поперечный момент свободной поверхности жидкости
    pub trans_moment_of_inertia: f64,
}
///
impl LiquidResult {
    ///
    pub fn new(
        assignment_id: usize,
        assigment_type: AssignmentType, 
        mass: f64,
        mass_shift: Position,
        long_moment_of_inertia: f64,
        trans_moment_of_inertia: f64,
    ) -> Self {
        Self {
            assignment_id,
            assigment_type, 
            mass,
            mass_shift,
            long_moment_of_inertia,
            trans_moment_of_inertia,
        }
    }
}
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct BulkResult {
    /// ID помещения
    pub code: String,  // TODO - убрать после переноса расчета момента в модель
    /// ID assigned
    pub assignment_id: usize,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,  
    /// Масса груза
    pub mass: f64,
    /// Смещение центра массы груза
    pub mass_shift: Position, 
    ///  Признак смещаемости груза
    pub shiftable: bool,     
    ///  Уровень заполнения отсека
    pub level: f64,
    /// Объем груза в отсеке
    pub volume: f64,
    /// Поперечный момент инерции площади ватерлинии относительно осей, параллельных осям X, м^4 
    pub inertia_trans_x: f64,    
    /// Продольный момент инерции площади ватерлинии относительно осей, параллельных осям Y, м^4 
    pub inertia_long_y: f64,  
    /// Максимальный поперечный момент инерции площади ватерлинии относительно осей, параллельных осям X, м^4 
    pub max_inertia_trans_x: f64,   
    /// Абсолютный момент жидкости при текущих углах и объеме
    pub grain_moment: f64,
}
///
impl BulkResult {
    ///
    pub fn new(
        code: String,
        assignment_id: usize,
        assigment_type: AssignmentType, 
        mass: f64,
        mass_shift: Position,
        shiftable: bool,
        level: f64,
        volume: f64,
        inertia_trans_x: f64,    
        inertia_long_y: f64, 
        max_inertia_trans_x: f64,   
    ) -> Self {
        Self {
            code,
            assignment_id,
            assigment_type, 
            mass,
            mass_shift,
            shiftable,
            level,
            volume,
            inertia_trans_x,    
            inertia_long_y, 
            max_inertia_trans_x,   
            grain_moment: 0.,
        }
    }
}
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct HoldCompartmentResult {  
    mass_shift: Position,
    level: f64,
    long_moment_of_inertia_max: f64,
    trans_moment_of_inertia_max: f64,  
}


