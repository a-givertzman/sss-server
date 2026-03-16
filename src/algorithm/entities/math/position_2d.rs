use bincode::{Decode, Encode};

///
/// Координата на 2D плоскости
#[derive(Debug, Copy, Clone, Decode, Encode, PartialEq)]
pub struct Position2d {
    x: f64,
    y: f64,
}
//
//
impl Position2d {
    /// Основной конструктор
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    //
    pub fn x(&self) -> f64 {
        self.x
    }
    //
    pub fn y(&self) -> f64 {
        self.y
    }
}