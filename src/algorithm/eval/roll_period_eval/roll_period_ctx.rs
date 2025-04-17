//! Результаты расчета периода качки судна  
#[derive(Debug, Clone)]
pub struct RollingPeriodCtx {
    /// Коэффициент для расчета периода
    pub c: f64,
    /// Период качки судна
    pub roll_period: f64,
}
