//! Кривая, позволяет получать интерполированные значения
use super::Value;
use sal_core::error::Error;
use splines::{Interpolation, Key, Spline};
use std::ops::{Add, Sub};

/// Представление кривой в виде массива пар значений
/// - Обеспечивает получение промежуточных значений с помощью простой линейной интерполяции
#[derive(Clone, Debug)]
pub struct Curve<T>
where
    T: Value + Add<T, Output = T> + Sub<T, Output = T>,
{
    spline: Spline<f64, T>,
}
//
impl<T> Curve<T>
where
    T: Value + Add<T, Output = T> + Sub<T, Output = T>,
{
    /// Creates new instance of the Curve with linear interpolation  
    /// from vector of the key - value pairs
    pub fn new_linear(src: &[(f64, T)]) -> Result<Curve<T>, Error> {
        if src.len() <= 1 {
            return Err(Error::new("Curve", "new_linear").err("src.len() <= 1"));
        }
        let src: Vec<_> = src
            .iter()
            .map(|v| Key::new(v.0, v.1, Interpolation::Linear))
            .collect();
        Ok(Self {
            spline: Spline::from_vec(src),
        })
    }
}

impl<T> ICurve<T> for Curve<T>
where
    T: Value + Add<T, Output = T> + Sub<T, Output = T>,
{
    /// Возвращает значение из таблицы по его ключу
    /// - если такого ключа нет, то возвращает промежуточное значение между двумя соседними с помощью линейной интерполяции
    /// - если ключ за пределами ключей таблицы, то вернет либо первое либо последнее значение
    /// - panic - если нет ключей
    fn value(&self, key: f64) -> Result<T, Error> {
        let res = self.spline.clamped_sample(key).ok_or(format!(
            "Curve value spline.clamped_sample(key) error: key:{key} spline:{:?}",
            self.spline
        ))?;
        //    log::info!("\t Curve clamped_value key:{key} res:{res}");
        Ok(res)
    }
    /// Численное интегрирование методом трапеций
    fn integral(&self, start: f64, end: f64) -> Result<T, Error> {
        if start > end {
            return Err(Error::new("Curve", "integral").err("start > end"));
        }
        if start == end {
            return Ok(T::zero());
        }
        let mut sum = T::zero();
        let n = 100;
        let delta = (end - start) / n as f64;
        let mut last_value = self
            .value(start)
            .map_err(|e| Error::new("Curve", "integral").err(format!("last_value error: {}", e)))?;
        let mut key = start;
        for _ in 0..n {
            key += delta;
            let next_value = self.value(key).map_err(|e| {
                Error::new("Curve", "integral").err(format!("next_value error: {}", e))
            })?;
            sum += (last_value + next_value).multiple(delta / 2.);
            last_value = next_value;
        }
        Ok(sum)
    }
}

#[doc(hidden)]
///
/// Interface used for testing purposes only
pub trait ICurve<T>
where
    T: Value + Add<T, Output = T> + Sub<T, Output = T>,
{
    fn value(&self, _: f64) -> Result<T, Error>;
    fn integral(&self, start: f64, end: f64) -> Result<T, Error>;
}
#[doc(hidden)]
// заглушка для тестирования
pub struct FakeCurve {
    value: f64,
    integral: f64,
}
#[doc(hidden)]
#[allow(dead_code)]
impl FakeCurve {
    pub fn new(value: f64, integral: f64) -> Self {
        Self { value, integral }
    }
}
#[doc(hidden)]
impl ICurve<f64> for FakeCurve {
    fn value(&self, _: f64) -> Result<f64, Error> {
        Ok(self.value)
    }
    fn integral(&self, _: f64, _: f64) -> Result<f64, Error> {
        Ok(self.integral)
    }
}
