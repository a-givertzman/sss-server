//! Непрерывный набор диапазонов значений

use bincode::{Decode, Encode};
use sal_core::error::Error;

use super::Bound;
/// Непрерывный набор диапазонов значений
#[derive(Debug, Clone, PartialEq, Decode, Encode)]
pub struct Bounds {
    // Непрерывный вектор диапазонов
    values: Vec<Bound>,
}
//
impl Bounds {
    /// Основной конструктор
    pub fn new(values: Vec<Bound>) -> Result<Self, Error> {
        let error = Error::new("Bounds", "new");
        for v in &values {
            match v {
                Bound::None => return Err(error.err("Bound::None in values")),
                Bound::Full => return Err(error.err("Bound::Full in values")),
                Bound::Value(_, _) => continue,
            }
        }
        if values.len() < 2 {
            return Err(error.err("values.len() < 2 "));
        }
        Ok(Self { values })
    }
    /// Вспомогательный конструктор
    /// * loa - L.O.A
    /// * middle_x - X midship from Fr0
    /// * n - Number of Parts
    #[allow(unused)]
    pub fn from_n(loa: f64, middle_x: f64, n: usize) -> Result<Self, Error> {
        let error = Error::new("Bounds", "from_n");
        if loa <= 0. {
            return Err(error.err(format!("loa {loa} <= 0.")));
        }
        if n <= 1 {
            return Err(error.err(format!("n {n} <= 1")));
        }
        let n_parts = n as f64;
        let mut values = Vec::new();
        for i in 0..n {
            let i = i as f64;
            values.push(
                Bound::new(
                    loa * i / n_parts - middle_x,
                    loa * (i + 1.) / n_parts - middle_x,
                )
                .map_err(|e| error.pass_with("Bound::new", e))?,
            );
        }
        Self::new(values)
    }
    /// Вспомогательный конструктор
    #[allow(unused)]
    pub fn from_min_max(min: f64, max: f64, n: usize) -> Result<Self, Error> {
        let error = Error::new("Bounds", "from_min_max");
        if min >= max {
            return Err(error.err(format!("min {min} >= max {max}")));
        }
        if n <= 1 {
            return Err(error.err(format!("n {n} <= 1")));
        }
        let n_parts = n as f64;
        let len = max - min;
        let mut values = Vec::new();
        for i in 0..n {
            let i = i as f64;
            values.push(
                Bound::new(len * i / n_parts + min, len * (i + 1.) / n_parts + min)
                    .map_err(|e| error.pass_with("Bound::new", e))?,
            );
        }
        Self::new(values)
    }
    /// Вспомогательный конструктор
    pub fn from_frames(frames: &[(f64, f64)]) -> Result<Self, Error> {
        let error = Error::new("Bounds", "from_min_max");
        if frames.len() <= 1 {
            return Err(error.err("frames.len() <= 1"));
        }
        let mut values = Vec::new();
        for frame in frames {
            values
                .push(Bound::new(frame.0, frame.1).map_err(|e| error.pass_with("Bound::new", e))?);
        }
        log::trace!(
            "Bounds.from_frames | frames:{:?} values:{:?} ",
            frames,
            values
        );
        Self::new(values)
    }
    /// Итератор по коллекции
    pub fn iter(&self) -> std::slice::Iter<'_, Bound> {
        self.values.iter()
    }
    /// Данные коллекции
    pub fn data(self) -> Vec<Bound> {
        self.values
    }
    /// Длинна диапазона
    #[allow(unused)]
    pub fn length(&self) -> f64 {
        self.values
            .last()
            .expect("Bounds length error: no values!")
            .end()
            .expect("Bounds delta error: no end value for last element!")
            - self
                .values
                .first()
                .expect("No values!")
                .start()
                .expect("Bounds delta error: no start value for first element!")
    }
    /// Количество разбиений
    #[allow(unused)]
    pub fn len_qnt(&self) -> usize {
        self.values.len()
    }
    /// Длинна элемента разбиения
    pub fn delta(&self) -> f64 {
        self.values
            .first()
            .expect("Bounds delta error: no values!")
            .length()
            .expect("Bounds delta error: no length for first element!")
    }
    /// Преобразование диапазона значений
    /// Возвращает вектор значений, пересчитанный к дипазону
    pub fn intersect(&self, bounds: &Bounds, values: &[f64]) -> Result<Vec<f64>, Error> {
        let error = Error::new("Bounds", "intersect");
        let bounds = bounds.iter();
        if bounds.len() != values.len() {
            return Err(error.err("bounds.len() != values.len()"));
        }
        let q_v: Vec<_> = bounds.zip(values.iter()).collect();
        let s_v = &self.values;
        let (mut q_i, mut s_i) = (0, 0);
        let mut current_q_i = None;
        let mut result = Vec::new();
        while s_i < s_v.len() {
            result.push(0.);
            while q_i < q_v.len() {
                let q_b = q_v[q_i].0;
                let v = q_v[q_i].1;
                let s_b = &s_v[s_i];
                let part_ratio = q_b.part_ratio(s_b).map_err(|err| {
                    error.pass_with(
                        format!("q_b.part_ratio(s_b), query_b:{q_b}, self_b:{s_b}, current_i:{s_i}"),
                        err,
                    )
                })?;
                if current_q_i.is_some() && part_ratio == 0. {
                    break;
                } 
                result[s_i] += v * part_ratio; 
                if current_q_i.is_none() {
                    if part_ratio < 1. {
                        current_q_i = Some(q_i);
                    }
                }
                q_i += 1;
            }
            if current_q_i.is_some() {
                q_i = current_q_i.unwrap();
            }
            current_q_i = None;
            s_i += 1;
        }
        Ok(result)
    }
}
