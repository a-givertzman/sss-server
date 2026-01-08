///
/// Учет обледенения судна для расмчета прочности
#[derive(Debug, Clone, PartialEq)]
pub struct IcingStrCtx {
    /// Распределение массы по вектору разбиения
    pub mass_values: Vec<f64>,
}
