///
/// Учет обледенения судна для расчета прочности
#[derive(Debug, Clone, PartialEq)]
pub struct IcingStrCtx {
    /// Распределение массы по вектору разбиения
    pub mass_values: Vec<f64>,
}
