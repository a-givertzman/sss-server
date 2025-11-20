//! Результаты расчета плеча кренящего момента от давления ветра
#[derive(Debug, Clone)]
pub struct WindCtx {
    /// Плечо кренящего момента постоянного ветра
    pub arm_wind_static: f64, 
    /// Плечо кренящего момента порыва ветра
    pub arm_wind_dynamic: f64,
}
