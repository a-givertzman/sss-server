//! Изгибающий момент
#[derive(Debug, Clone)]
pub struct BendingMomentCtx {
    /// Распределение момента по вектору разбиения
    values: Vec<f64>,
}
//
impl BendingMomentCtx {
    /// Основной конструктор
    pub fn new(
        values: Vec<f64>
    ) -> Self {
        Self {
            values,
        }
    }
}
