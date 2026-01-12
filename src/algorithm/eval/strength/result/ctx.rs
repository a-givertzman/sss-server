//! Результаты расчета по прочности

use crate::algorithm::eval::strength::result::Results;
#[derive(Debug, Clone)]
pub struct ResultStrCtx {
    pub results: Results,
}
//
impl ResultStrCtx {
    /// Основной конструктор
    pub fn new(
        results: Results
    ) -> Self {
        Self {
            results,
        }
    }
}
