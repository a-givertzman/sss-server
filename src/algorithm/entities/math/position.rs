//! Точка относительно Центра
use std::{
    iter::Sum,
    ops::{Add, AddAssign, Sub},
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
//
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Decode, Encode, PartialEq, Default)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}
//
impl Position {
    /// Основной конструктор
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    //
    pub fn x(&self) -> f64 {
        self.x
    }
    //
    pub fn y(&self) -> f64 {
        self.y
    }
    //
    pub fn z(&self) -> f64 {
        self.z
    }
    //
    pub fn values(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }   
    //
    #[allow(unused)]
    pub fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    //
    pub fn print(&self) -> String {
        format!("({:.3} {:.3} {:.3})", self.x, self.y, self.z)
    }
}
//
impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Position({}, {}, {})", self.x(), self.y(), self.z())
    }
}
//
impl Add for Position {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Position::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}
//
impl Sub for Position {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Position::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}
//
impl Sum for Position {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(0., 0., 0.), |a, b| a + b)
    }
}
//
impl AddAssign for Position {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        };
    }
}
//
impl From<Position> for [f64; 3] {
    fn from(val: Position) -> Self {
        [val.x, val.y, val.z]
    }
}
//
impl From<Position> for nalgebra::Point3<f64> {
    fn from(val: Position) -> Self {
        nalgebra::Point3::new(val.x, val.y, val.z)
    }
}
//
impl From<nalgebra::Point3<f64>> for Position {
    fn from(v: nalgebra::Point3<f64>) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}
//
impl From<(f64, f64, f64)> for Position {
    fn from(v: (f64, f64, f64)) -> Self {
        Self::new(v.0, v.1, v.2)
    }
}

