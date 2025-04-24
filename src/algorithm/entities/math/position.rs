//! Точка относительно Центра
use std::{
    iter::Sum,
    ops::{Add, AddAssign, Sub},
};
use bincode::{Decode, Encode};
use serde::Deserialize;
//
#[derive(Debug, Copy, Clone, Deserialize, Encode, PartialEq)]
pub struct Point3 {
    x: f64,
    y: f64,
    z: f64,
}
//
impl TryFrom<Point3> for Position {
    type Error = String;
    fn try_from(data: Point3) -> Result<Self, Self::Error> {
        Ok(Position {
            x: data.x,
            y: data.y,
            z: data.z,
        })
    }
}
//
#[derive(Debug, Copy, Clone, Deserialize, Decode, Encode, PartialEq)]
#[serde(try_from = "Point3")]
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
    /// Дополнительный конструктор  
    /// * (f64, f64, f64) - x, y, z
    #[allow(unused)]
    pub fn from(v: (f64, f64, f64)) -> Self {
        Self::new(v.0, v.1, v.2)
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
    #[allow(unused)]
    pub fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
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
