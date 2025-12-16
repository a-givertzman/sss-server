use crate::algorithm::entities::{Moment, Position};

///
/// Распределение по шпациям площади горизонтальных поверхностей и
/// площади парусности судна
#[derive(Debug, Clone)]
pub struct StaticAreaCtx {
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
    /// Суммарная площадь обледенения
    pub area_timber_icing_h: f64,
    /// Смещение центра площади обледенения
    pub area_timber_icing_h_shift: Position,
    /// Распределение площади по шпациям
    pub area_timber_icing_h_values: Vec<f64>,
    // Момент площади обледенения палубного груза - леса
    pub icing_timber_moment: Moment,
    // Изменение момента горизонтальной площади обледенения палубного груза - леса
    // относительно палубы
    pub icing_delta_timber_moment: Moment,
}
