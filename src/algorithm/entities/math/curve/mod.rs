use std::ops::AddAssign;

mod curve1d;
mod curve2d;
mod curve3d;

pub use curve1d::*;
pub use curve2d::*;
pub use curve3d::*;

use splines::Interpolate;

use super::Position;


pub trait Zero {
    fn zero() -> Self;
}

pub trait Mulf64 {
    fn multiple(&self, rhs: f64) -> Self;
}

pub trait Value: Sized + Copy + Zero + AddAssign + Mulf64 + Interpolate<f64> + std::fmt::Debug {}

//
#[doc(hidden)]
impl Mulf64 for f64 {
    fn multiple(&self, rhs: f64) -> Self {
        self*rhs
    }
}
//
#[doc(hidden)]
impl Zero for f64 {
    fn zero() -> Self {
        0.
    }
}
//
#[doc(hidden)]
impl Value for f64 {}
//
#[doc(hidden)]
impl Mulf64 for Position {
    fn multiple(&self, rhs: f64) -> Self {
        self.scale(rhs)
    }
}
//
#[doc(hidden)]
impl Zero for Position {
    fn zero() -> Self {
        Position::zero()
    }
}
//
impl Value for Position {}
//
#[doc(hidden)]
impl Interpolate<f64> for Position {
    fn step(t: f64, threshold: f64, a: Self, b: Self) -> Self {
        Position::new(
            f64::step(t, threshold, a.x(), b.x()),
            f64::step(t, threshold, a.y(), b.y()),
            f64::step(t, threshold, a.z(), b.z()),
        )
    }

    fn lerp(t: f64, a: Self, b: Self) -> Self {
        Position::new(
            f64::lerp(t,  a.x(), b.x()),
            f64::lerp(t,  a.y(), b.y()),
            f64::lerp(t,  a.z(), b.z()),
        )
    }

    fn cosine(t: f64, a: Self, b: Self) -> Self {
        Position::new(
            f64::cosine(t,  a.x(), b.x()),
            f64::cosine(t,  a.y(), b.y()),
            f64::cosine(t,  a.z(), b.z()),
        )
    }

    fn cubic_hermite(t: f64, x: (f64, Self), a: (f64, Self), b: (f64, Self), y: (f64, Self)) -> Self {
        Position::new(
            f64::cubic_hermite(t,  (x.0, x.1.x()), (a.0, a.1.x()), (b.0, b.1.x()), (y.0, y.1.x())),
            f64::cubic_hermite(t,  (x.0, x.1.y()), (a.0, a.1.y()), (b.0, b.1.y()), (y.0, y.1.y())),
            f64::cubic_hermite(t,  (x.0, x.1.z()), (a.0, a.1.z()), (b.0, b.1.z()), (y.0, y.1.z())),
        )
    }

    fn quadratic_bezier(t: f64, a: Self, u: Self, b: Self) -> Self {
        Position::new(
            f64::quadratic_bezier(t,  a.x(), u.x(), b.x()),
            f64::quadratic_bezier(t,  a.y(), u.y(), b.y()),
            f64::quadratic_bezier(t,  a.z(), u.z(), b.z()),
        )
    }

    fn cubic_bezier(t: f64, a: Self, u: Self, v: Self, b: Self) -> Self {
        Position::new(
            f64::cubic_bezier(t,  a.x(), u.x(), v.x(), b.x()),
            f64::cubic_bezier(t,  a.y(), u.y(), v.y(), b.y()),
            f64::cubic_bezier(t,  a.z(), u.z(), v.z(), b.z()),
        )
    }

    fn cubic_bezier_mirrored(t: f64, a: Self, u: Self, v: Self, b: Self) -> Self {
        Position::new(
            f64::cubic_bezier_mirrored(t,  a.x(), u.x(), v.x(), b.x()),
            f64::cubic_bezier_mirrored(t,  a.y(), u.y(), v.y(), b.y()),
            f64::cubic_bezier_mirrored(t,  a.z(), u.z(), v.z(), b.z()),
        )
    }
}
//
pub struct CurveResult<T>
where
    T: Value
{
    pub value: T,
    pub is_clamped: bool,
}
//
impl<T> CurveResult<T>
where
    T: Value 
{
    //
    pub fn new(value: T, is_clamped: bool) -> Self {
        Self{value, is_clamped}
    }
}
