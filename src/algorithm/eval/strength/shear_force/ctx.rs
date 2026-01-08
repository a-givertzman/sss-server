//! Срезающая сила, действующая на корпус судна
#[derive(Debug, Clone)]
pub struct ShearForceCtx {
    /// Распределение силы по вектору разбиения
    pub values: Vec<f64>,
}
//
impl ShearForceCtx {
    /// Основной конструктор
    pub fn new(
        values: Vec<f64>
    ) -> Self {
        Self {
            values,
        }
    }
}
