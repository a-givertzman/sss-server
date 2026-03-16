//! Результаты расчета парусности судна
#[derive(Debug, Clone)]
pub struct WindageCtx {
    /// Площадь парусности, м^2
    pub a_v: f64,
    /// Плечо парусности, м
    pub z_v: f64,
}
