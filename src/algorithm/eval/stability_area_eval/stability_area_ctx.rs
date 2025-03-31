use crate::algorithm::entities::Position;

/// Площади поверхностей и их центры
#[derive(Debug, Clone)]
pub struct StabilityAreaCtx {
    /// Площадь парусности корпуса
    pub const_area_v: Vec<(f64, Position)>,
    /// Площадь горизонтальных поверхностей открытых палуб
    pub const_area_h: Vec<(f64, Position)>,
    /// Площадь парусности палубного груза
    pub unit_area_v: Vec<(f64, f64, Position)>,   // area, h, center of area
    /// Площадь горизонтальных поверхностей палубного груза
    pub unit_area_h: Vec<(f64, Position)>,        // area, h, center of area
    /// Площадь горизонтальных поверхностей палубного лесного груза
    pub unit_area_timber_h: Vec<(f64, Position)>,
}
