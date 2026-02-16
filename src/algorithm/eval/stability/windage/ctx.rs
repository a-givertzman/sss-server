//! Результаты расчета парусности судна
#[derive(Debug, Clone)]
pub struct WindageCtx {
    /// Площадь парусности, м^2
    pub av: f64,
    /// Плечо парусности по x, м
    pub xv: f64,
    /// Плечо парусности по z, м
    pub zv: f64,
}
