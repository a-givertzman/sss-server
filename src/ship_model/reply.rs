use sal_core::error::Error;

use crate::algorithm::entities::{parameters::ParameterID, Bounds, Position};
///
/// Type doc here
#[derive(Debug, Clone)]
pub struct LiquidData {
    /// ID груза
    pub cargo_id: usize,
    /// ID помещения
    pub space_id: usize,
    /// смещение центра массы
    pub mass_shift: Position,
    /// продольный момент свободной поверхности жидкости
    pub long_moment_of_inertia: f64,
    /// поперечный момент свободной поверхности жидкости
    pub trans_moment_of_inertia: f64,
    /// Распределение массы по шпациям, (index, value)
    pub mass_values: Vec<(usize, f64)>,
}
///
/// Type doc here
#[derive(Debug, Clone)]
pub struct BulkData {
    /// ID груза
    pub cargo_id: usize,
    /// ID помещения
    pub space_id: usize,
    /// смещение центра массы
    pub mass_shift: Position,
    /// Распределение массы по шпациям, (index, value)
    pub mass_values: Vec<(usize, f64)>,
}
///
/// Структура результатов расчета баланса судна
#[derive(Debug, Clone)]
pub struct BalanceResultData {
    // Результаты расчета в виде (id, value)
    // id в соответствии с https://github.com/a-givertzman/sss/blob/35-shipmodel-fix-unit-cargo/docs/user-guide/ru/part08_stability/chapter03_parametresStability.md
    pub parameters: Vec<(ParameterID, f64)>, 
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса.
    pub bulk: Vec<BulkData>,
    pub liquid: Vec<LiquidData>,
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
    /// Объемное водоизмещение
    pub volume: f64,
}
///
/// Replies from the `ShipModel`
#[derive(Debug)]
pub enum Reply {
    Bounds(Bounds),
    BoundAreas(Result<(Vec<f64>, Vec<f64>), Error>),
    ComputeBalance(Result<BalanceResultData, Error>),
}
