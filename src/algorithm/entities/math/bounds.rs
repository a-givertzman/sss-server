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
    /// Вспомогательный конструктор
    pub fn from_array(array: &[f64], shift: f64) -> Result<Self, Error> {
        let error = Error::new("Bounds", "from_array");
        if array.len() <= 1 {
            return Err(error.err("array.len() <= 1"));
        }
        let mut last = array[0];
        let frames: Vec<_> = (1..array.len()).map(|i| {
            let res = (last - shift, array[i] - shift);
            last = array[i];
            res
        }).collect();
        Self::from_frames(&frames)
    }
    /// Итератор по коллекции
    pub fn iter(&self) -> std::slice::Iter<'_, Bound> {
        self.values.iter()
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
 /*   /// Длинна элемента разбиения
    pub fn delta(&self) -> f64 {
        self.values
            .first()
            .expect("Bounds delta error: no values!")
            .length()
            .expect("Bounds delta error: no length for first element!")
    }*/
    /// Преобразование диапазона значений
    /// Возвращает вектор значений values в распределении bounds, пересчитанный к распределению self
    pub fn intersect(&self, src_bounds: &Bounds, src_values: &[f64]) -> Result<Vec<f64>, Error> {
        let error = Error::new("Bounds", "intersect");
        let bounds = src_bounds.iter();
        if bounds.len() != src_values.len() {
            return Err(error.err("bounds.len() != values.len()"));
        }
        let query_data: Vec<_> = bounds.zip(src_values.iter()).collect();
        let self_bounds = &self.values;
        let (mut query_index, mut self_index) = (0, 0);
        let mut current_q_i = None;
        let mut result = Vec::new();
        while self_index < self_bounds.len() {
            result.push(0.);
            while query_index < query_data.len() {
                let (query_bound, query_value) = query_data[query_index];
                let self_bound = &self_bounds[self_index];
                let part_ratio = query_bound.part_ratio(self_bound).map_err(|err| {
                    error.pass_with(
                        format!("q_b.part_ratio(s_b), query_bound:{query_bound}, self_bound:{self_bound}, current_i:{self_index}"),
                        err,
                    )
                })?;
                if part_ratio > 0. {
                    current_q_i = Some(query_index);
                    result[self_index] += query_value * part_ratio;
                } else if current_q_i.is_some() {
                    break;
                } 
                /*println!("self: i:{self_index} b:({:.3} {:.3}) query: i:{query_index} b:({:.3} {:.3}) v:{:.3}  pr:{:.3} res:{:.3} cqi:{:?}", 
                self_bound.start().unwrap_or(-10000.), self_bound.end().unwrap_or(-10000.), 
                query_bound.start().unwrap_or(-10000.), query_bound.end().unwrap_or(-10000.), query_value,
                part_ratio, result[self_index], current_q_i);*/
                query_index += 1;
            }
            query_index = current_q_i.unwrap_or(0);
            current_q_i = None;
            self_index += 1;
        }
        Ok(result)
    }
}
