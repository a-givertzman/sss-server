//! Промежуточные структуры для serde_json для парсинга данных объемного кренящего момента для зерна
use crate::algorithm::entities::{Curve, ICurve as _, data::DataArray};
use sal_core::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Данные по шпангоуту
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrainMomentData {
    /// ID помещения
    pub code: String,
    /// Уровень заполнения отсека
    pub level: f64,
    /// Объемный кренящий момент
    pub moment: f64,
}
//
impl std::fmt::Display for GrainMomentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GrainMomentData(code:{}, level:{}, moment:{} )",
            self.code, self.level, self.moment,
        )
    }
}
pub type GrainMomentDataArray = DataArray<GrainMomentData>;
//
impl GrainMomentDataArray {
    /// Преобразование и возвращает данные в виде мапы
    pub fn data(self) -> HashMap<String, Vec<(f64, f64)>> {
        let mut map: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
        self.data.into_iter().for_each(|v| {
            if let Some(vector) = map.get_mut(&v.code) {
                vector.push((v.level, v.moment));
            } else {
                map.insert(v.code, vec![(v.level, v.moment)]);
            }
        });
        map
    }
}
/// Класс, инкапсулирующий кривую зернового момента для отсека.
/// Может состоять из нескольких кривых
pub struct GrainMoment {
    curves: Vec<Curve<f64>>,
}///
impl GrainMoment {
    //
    pub fn new(curves: Vec<Curve<f64>>) -> Self {
        Self { curves }
    }
    //
    pub fn value(&self, key: f64) -> Result<f64, Error> {
        let (values, errors): (Vec<_>, Vec<_>) = self
            .curves
            .iter()
            .map(|c| c.value(key))
            .partition(|v| v.is_ok());
        if !errors.is_empty() {
            return Err(Error::new("GrainMoment", "value").pass(
                errors.iter().fold(String::new(), |acc, err| {
                    format!("{acc}\n\t error: {:?}", err)
                }),
            ));
        }
        Ok(values.into_iter().flatten().sum())
    }
    //
    pub fn curves(&self) -> Vec<Curve<f64>> {
        self.curves.clone()
    }
}
