//! Кривая поверхность, позволяет получать интерполированные значения
use std::ops::{Add, Sub};
use crate::kernel::error::error::Error;
use super::{Curve2D, CurveResult, ICurve2D, Value};
/// Представление поверхности в виде массива кривых пар значений
/// - Обеспечивает получение промежуточных значений с помощью простой линейной интерполяции
#[derive(Clone)]
pub struct Curve3D<T> where T: Value + Add<T, Output = T> + Sub <T, Output = T> {
    curves: Vec<(f64, Curve2D<T>)>,
}
type Values<T> = [(f64, Vec<(f64, Vec<(f64, T)>)>)];
//
impl<T> Curve3D<T> where T: Value + Add<T, Output = T> + Sub <T, Output = T>  {
    /// Основной конструктор
    #[allow(dead_code)]
    pub fn new(curves: Vec<(f64, Curve2D<T>)>) -> Result<Self, Error> {
        if curves.is_empty() {
            return Err(Error::FromString("Curve3D new error: curves.is_empty()".to_string()));
        }
        Ok(Self { curves })
    }
    /// Конструктор из матрицы значений,
    /// создает кривые с линейным методом интерполяции
    #[allow(dead_code)]
    pub fn from_values_linear(values: &Values<T>) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::FromString("Curve3D from_values_linear error: values.is_empty()".to_string()));
        }
    //    log::info!("\t Curve3D from_values_linear begin: values.len:{}", values.len());
        let mut curves = Vec::new();
        for (value, vector) in values.iter() {
    //        log::info!("\t Curve3D from_values_linear value{value}: values:{:?}", vector);
            curves.push((*value, Curve2D::from_values_linear(vector)?));     
        }
        curves.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Self::new(curves)
    }
    /// Конструктор из матрицы значений,
    /// создает кривые с Катмулла – Рома методом интерполяции
    #[allow(dead_code)]
    pub fn from_values_catmull_rom(values: &Values<T>) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::FromString("Curve3D from_values_catmull_rom error: values.is_empty()".to_string()));
        }
        let mut curves = Vec::new();
        for (value, vector) in values.iter() {
            curves.push((*value, Curve2D::from_values_catmull_rom(vector).map_err(|e| Error::FromString(format!("Curve3D from_values_catmull_rom error: {e}, value:{value}, vector:{:?}", vector)))?));     
        }
        Self::new(curves)
    }
    /// Конструктор из матрицы значений,
    /// создает кривые с косинусным методом интерполяции
    #[allow(dead_code)]
    pub fn new_cosine(values: &Values<T>) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::FromString("Curve3D new_cosine error: values.is_empty()".to_string()));
        }
        let mut curves = Vec::new();
        for (value, vector) in values.iter() {
            curves.push((*value, Curve2D::new_cosine(vector).map_err(|e| Error::FromString(format!("Curve3D from_values_catmull_rom error: {e}, value:{value}, vector:{:?}", vector)))?));     
        }
        Self::new(curves)
    }
}
//
impl<T> ICurve3D<T> for Curve3D<T> where T: Value + Add<T, Output = T> + Sub <T, Output = T>  {
    /// Возвращает значение из таблицы по его ключу
    /// - если такого ключа нет, то возвращает промежуточное значение между двумя соседними с помощью линейной интерполяции
    /// - если ключ за пределами ключей таблицы, то вернет либо первое либо последнее значение
    /// - panic - если нет ключей
    ///
    fn value(&self, key1: f64, key2: f64, key3: f64) -> Result<CurveResult<T>, Error> {
    //    log::info!("\t Curve3D curve begin: key1:{key1} key2:{key2}");
        for index in 0..self.curves.len() {
            if self.curves[index].0 >= key1 {
                if index == 0 {
                    let result = self.curves[0].1.value(key2, key3)?;
    //                log::info!("\t Curve3D curve index = 0, value={:?}", res);
                    return Ok(CurveResult::new(result.value, true));//self.curves[0].1.value(key2);
                }
                let res1 = self.curves[index - 1].1.value(key2, key3)?;
                let res2 = self.curves[index].1.value(key2, key3)?;
                let is_clamped = res1.is_clamped || res2.is_clamped;
                let delta = self.curves[index].0 - self.curves[index - 1].0;
                let coeff1 = (self.curves[index].0 - key1) / delta;
                let coeff2 = 1. - coeff1;
                let res1 = res1.value.multiple(coeff1);
                let res2 = res2.value.multiple(coeff2);
                let result= res1 + res2;
    //            log::info!("\t Curve3D value key1:{key1} key2:{key2} key3:{key3} res1:{:?} res2:{:?} delta:{delta} coeff1:{coeff1} coeff2:{coeff2} result:{:?}", res1, res2, result);
                return Ok(CurveResult::new(result, is_clamped));
            }
        }
        let result = self
            .curves
            .last()
            .ok_or("Curve3D value error: no last curve".to_string())?
            .1
            .value(key2, key3)?;
        Ok(CurveResult::new(result.value, true))
    }
}
//
#[doc(hidden)]
/// Interface used for testing purposes only
pub trait ICurve3D<T> where T: Value + Add<T, Output = T> + Sub <T, Output = T> {
    fn value(&self, key1: f64, key2: f64, key3: f64) -> Result<CurveResult<T>, Error>;
}
#[doc(hidden)]
// заглушка для тестирования
pub struct FakeCurve3D {
    value: f64,
}
//
#[doc(hidden)]
#[allow(dead_code)]
impl FakeCurve3D {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}
//
#[doc(hidden)]
impl ICurve3D<f64> for FakeCurve3D {
    fn value(&self, _: f64, _: f64, _: f64) -> Result<CurveResult<f64>, Error> {
        Ok(CurveResult::new(self.value, false))
    }
}
