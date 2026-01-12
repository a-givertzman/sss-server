//! Результаты расчета по прочности
pub mod ctx;
pub mod eval;

use std::{cell::RefCell, f64};
/// Набор результатов расчетов для записи в БД
#[derive(Debug, Clone)]
pub struct Results {
    values: RefCell<(Vec<String>, Vec<Vec<f64>>)>,
    results: RefCell<(Vec<String>, Vec<Vec<f64>>)>,
}
//
impl Results {
    pub fn new() -> Self {
        Self {
            values: RefCell::new((Vec::new(), Vec::new())),
            results: RefCell::new((Vec::new(), Vec::new())),
        }
    }
}
//
impl IResults for Results {
    /// Добавление данных промежуточных значений
    fn add_values(&self, name: &str, values: &Vec<f64>) {
        let mut data = self.values.borrow_mut();
        data.0.push(name.to_owned());
        let base_vec = &mut data.1;
        if base_vec.is_empty() {
            values.iter().for_each(|&v| base_vec.push(vec![v]));
        } else {
            base_vec
                .iter_mut()
                .enumerate()
                .for_each(|(i, vector)| vector.push(values[i]));
        }
    }
    /// Добавление рассчитанных данных силы и момента
    fn add_results(&self, name: &str, values: &Vec<f64>) {
        let mut data = self.results.borrow_mut();
        data.0.push(name.to_owned());
        let base_vec = &mut data.1;
        if base_vec.is_empty() {
            values.iter().for_each(|&v| base_vec.push(vec![v]));
        } else {
            base_vec
                .iter_mut()
                .enumerate()
                .for_each(|(i, vector)| vector.push(values[i]));
        }
    }
    /// Геттер для данных
    fn take_values(&self) -> (Vec<String>, Vec<Vec<f64>>) {
        self.values.take()
    }
    /// Геттер для результатов
    fn take_results(&self) -> (Vec<String>, Vec<Vec<f64>>) {
        self.results.take()
    }
}
#[doc(hidden)]
pub trait IResults {
    /// Добавление данных промежуточных значений
    fn add_values(&self, name: &str, values: &Vec<f64>);
    /// Добавление рассчитанных данных силы и момента
    fn add_results(&self, name: &str, values: &Vec<f64>);
    /// Получение данных промежуточных значений
    fn take_values(&self) -> (Vec<String>, Vec<Vec<f64>>);
    /// Получение рассчитанных данных силы и момента
    fn take_results(&self) -> (Vec<String>, Vec<Vec<f64>>);
}
// заглушка для тестирования
#[doc(hidden)]
pub struct FakeResults;
#[doc(hidden)]
#[allow(dead_code)]
impl IResults for FakeResults {
    fn add_values(&self, _: &str, _: &Vec<f64>) {}
    fn add_results(&self, _: &str, _: &Vec<f64>) {}
    fn take_values(&self) -> (Vec<String>, Vec<Vec<f64>>) {
        (Vec::new(), Vec::new())
    }
    fn take_results(&self) -> (Vec<String>, Vec<Vec<f64>>) {
        (Vec::new(), Vec::new())
    }
}


