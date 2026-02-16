//! Результирующая нагрузка на шпацию
#[derive(Debug, Clone)]
pub struct TotalForceCtx {
    /// Распределение нагрузки по вектору разбиения
    pub values: Vec<f64>,
}
//
impl TotalForceCtx {
    /// Основной конструктор
    pub fn new(
        values: Vec<f64>
    ) -> Self {
        Self {
            values,
        }
    }
}
