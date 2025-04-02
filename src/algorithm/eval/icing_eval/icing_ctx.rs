use crate::algorithm::entities::Position;

///
/// Учет обледенения судна.
#[derive(Debug, Clone)]
pub struct IcingCtx {
    /// Суммарная масса
    pub mass_sum: f64,
    /// Смещение центра массы по оси Х
    pub mass_shift_x: f64,
    /// Распределение массы по вектору разбиения
    pub mass_values: Vec<f64>,
}
