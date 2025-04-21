use crate::algorithm::entities::Moment;

/// Площади поверхностей для расчета остойчивости
#[derive(Debug, Clone)]
pub struct StabilityAreaCtx {
    /// Площадь парусности
    pub area_v: f64, 
    /// Момент площади парусности
    pub moment_v: Moment,
    /// Момент площади горизонтальных поверхностей
    pub moment_h: Moment,
    /// Момент площади горизонтальных поверхностей палубного груза - леса
    pub moment_timber_h: Moment,
    /// Изменение момента площади горизонтальных поверхностей палубного груза - леса
    /// относительно палубы
    pub delta_moment_timber_h: Moment, 
}
