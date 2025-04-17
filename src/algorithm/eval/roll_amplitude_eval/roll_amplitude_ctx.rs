//! Результаты расчета амплитуды качки судна
#[derive(Debug, Clone)]
pub struct RollingAmplitudeCtx {
    /// Коэффициент для расчета периода
    pub c: f64,
    /// Период качки судна
    pub roll_period: f64, 
}
