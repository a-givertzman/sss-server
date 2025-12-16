use crate::algorithm::entities::Position;

///
/// Учет обледенения судна.
#[derive(Debug, Clone, PartialEq)]
pub struct IcingCtx {
    /// Суммарная масса
    pub mass: f64,
    /// Смещение центра массы
    pub mass_shift: Position,
    /// Распределение массы по вектору разбиения
    pub mass_values: Vec<f64>,
}
