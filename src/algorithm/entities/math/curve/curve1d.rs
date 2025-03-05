//! Кривая, позволяет получать интерполированные значения
use std::ops::{Add, Sub};

use splines::{Interpolation, Key, Spline};

use crate::kernel::error::error::Error;

use super::Value;

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
            return Err(Error::FromString(
                "Curve new_linear error: src.len() <= 1".to_string(),
            ));
        }
        let src: Vec<_> = src
            .iter()
            .map(|v| Key::new(v.0, v.1, Interpolation::Linear))
            .collect();
        Ok(Self {
            spline: Spline::from_vec(src),
        })
    }
    /// Creates new instance of the Curve with CatmullRom interpolation  
    /// from vector of the key - value pairs
    /// Values must be sorted by key
    pub fn new_catmull_rom(src: &[(f64, T)]) -> Result<Curve<T>, Error> {
        if src.len() <= 2 {
            return Err(Error::FromString(
                "Curve new_catmull_rom error: src.len() <= 2".to_string(),
            ));
        }
        let mut res = Vec::new();
        let mut src = Vec::from(src);
        src.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .expect("Curve.new_catmull_rom src sort error!")
        });
        // Для метода CatmullRom добавляем по 3 значения вначало и конец вектора
        let delta_key1 = src[1].0 - src[0].0;
        let delta_key2 = src[2].0 - src[1].0;
        if delta_key1 <= 0. || delta_key2 <= 0. {
            return Err(Error::FromString(
                "Curve new_catmull_rom error: delta_key <= 0.".to_string(),
            ));
        }
        let delta_value1 = (src[1].1 - src[0].1).multiple(1./ delta_key1);
        let delta_value2 = (src[2].1 - src[1].1).multiple(1./ delta_key2);
        let delta = (delta_value1 - delta_value2).multiple(1./ delta_key1);
        let mut add_value = |i: f64| {
            let delta_key = delta_key1 * i;
            res.push(Key::new(
                src[0].0 - delta_key,
                src[0].1 - (delta_value1 + delta.multiple(delta_key)).multiple(delta_key),
                Interpolation::CatmullRom,
            ));
        };
        add_value(3.);
        add_value(2.);
        add_value(1.);
        let values: Vec<Key<_, _>> = src
            .iter()
            .map(|v| Key::new(v.0, v.1, Interpolation::CatmullRom))
            .collect();
        res.append(&mut values.clone());

        let delta_key1 = src[src.len() - 1].0 - src[src.len() - 2].0;
        let delta_key2: f64 = src[src.len() - 2].0 - src[src.len() - 3].0;
        if delta_key1 <= 0. || delta_key2 <= 0. {
            return Err(Error::FromString(
                "Curve new_catmull_rom error: delta_key <= 0.".to_string(),
            ));
        }
        let delta_value1 = (src[src.len() - 1].1 - src[src.len() - 2].1).multiple(1./ delta_key1);
        let delta_value2 = (src[src.len() - 2].1 - src[src.len() - 3].1).multiple(1./ delta_key2);
        let delta = (delta_value1 - delta_value2).multiple(1./ delta_key1);
        let mut add_value = |i: f64| {
            let delta_key = delta_key1 * i;
            res.push(Key::new(
                src.last().unwrap().0 + delta_key,
                src.last().unwrap().1 + (delta_value1 + delta.multiple(delta_key)).multiple(delta_key),
                Interpolation::CatmullRom,
            ));
        };
        add_value(1.);
        add_value(2.);
        add_value(3.);
        Ok(Self {
            spline: Spline::from_vec(res),
        })
    }
    /// Creates new instance of the Curve with Cosine interpolation  
    /// from vector of the key - value pairs
    /// Values must be sorted by key
    pub fn new_cosine(src: &[(f64, T)]) -> Result<Curve<T>, Error> {
        if src.len() <= 2 {
            return Err(Error::FromString(
                "Curve new_cosine error: src.len() <= 2".to_string(),
            ));
        }
        let mut res = Vec::new();
        let mut src = Vec::from(src);
        src.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .expect("Curve.new_cosine src sort error!")
        });
        // Для метода CatmullRom добавляем по 3 значения вначало и конец вектора
        let delta_key = src[1].0 - src[0].0;
        if delta_key <= 0. {
            return Err(Error::FromString(
                "Curve new_cosine error: delta_key <= 0.".to_string(),
            ));
        }
        let delta_value = src[1].1 - src[0].1;
        res.push(Key::new(
            src[0].0 - delta_key,
            src[0].1 - delta_value,
            Interpolation::Cosine,
        ));
        let values: Vec<Key<_, _>> = src
            .iter()
            .map(|v| Key::new(v.0, v.1, Interpolation::Cosine))
            .collect();
        res.append(&mut values.clone());
        let delta_key = src[src.len() - 1].0 - src[src.len() - 2].0;
        if delta_key <= 0. {
            return Err(Error::FromString(
                "Curve new_cosine error: delta_key <= 0".to_string(),
            ));
        }
        let delta_value = src[src.len() - 1].1 - src[src.len() - 2].1;
        res.push(Key::new(
            src.last().unwrap().0 + delta_key,
            src.last().unwrap().1 + delta_value,
            Interpolation::Cosine,
        ));
        Ok(Self {
            spline: Spline::from_vec(res),
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
            return Err(Error::FromString(
                "Curve integral error: start > end".to_string(),
            ));
        }
        if start == end {
            return Ok(T::zero());
        }
        let mut sum = T::zero();
        let n = 100;
        let delta = (end - start) / n as f64;
        let mut last_value = self
            .value(start)
            .map_err(|e| Error::FromString(format!("Curve integral last_value error: {}", e)))?;
        let mut key = start;
        for _ in 0..n {
            key += delta;
            let next_value = self.value(key).map_err(|e| {
                Error::FromString(format!("Curve integral next_value error: {}", e))
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
