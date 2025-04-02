use crate::algorithm::entities::Position;

///
/// Распределение по шпациям площади горизонтальных поверхностей и
/// площади парусности судна
#[derive(Debug, Clone)]
pub struct StrengthAreaCtx {
    /// Площадь парусности
    /// Суммарная площадь
    pub area_v: f64,
    /// Смещение центра площади
    pub area_v_shift: Position,
    /// Распределение площади по шпациям
    pub area_v_values: Vec<f64>,
    /// Площадь горизонтальных поверхностей открытых палуб
    /// Суммарная площадь
    pub area_h: f64,
    /// Смещение центра площади
    pub area_h_shift: Position,
    /// Распределение площади по шпациям
    pub area_h_values: Vec<f64>,
    /// Площадь горизонтальных поверхностей палубного лесного груза
    /// Суммарная площадь
    pub area_timber_h: f64,
    /// Смещение центра площади
    pub area_timber_h_shift: Position,
    /// Распределение площади по шпациям
    pub area_timber_h_values: Vec<f64>,
}
