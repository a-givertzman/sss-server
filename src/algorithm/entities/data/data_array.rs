//! Промежуточные структуры для serde_json для парсинга данных судна
use crate::algorithm::entities::Position;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_parser::IFromJson;
//
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pair {
    pub key: f64,
    pub value: f64,
}
//
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Triple {
    pub key: f64,
    pub value_x: f64,
    pub value_y: f64,
}
//
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quadruple {
    pub key: f64,
    pub value_x: f64,
    pub value_y: f64,
    pub value_z: f64,
}
//
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrimVolumeData {
    pub trim: f64,
    pub volume: f64,
    pub value: f64,
}
//
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrimDraughtData {
    pub trim: f64,
    pub draught: f64,
    pub value: f64,
}
/// Массив ключ + значение
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataArray<T> {
    pub data: Vec<T>,
    pub error: HashMap<String, String>,
}
//
impl<T> IFromJson for DataArray<T> {
    fn error(&self) -> Option<&String> {
        self.error.values().next()
    }
}
//
impl DataArray<Pair> {
    /// Преобразование данных в массив ключ + значение
    pub fn data(&self) -> Vec<(f64, f64)> {
        self.data.iter().map(|v| (v.key, v.value)).collect()
    }
}
//
#[allow(dead_code)]
impl DataArray<Triple> {
    /// Преобразование данных в массив ключ + значение по х
    pub fn x(&self) -> Vec<(f64, f64)> {
        self.data.iter().map(|v| (v.key, v.value_x)).collect()
    }
    /// Преобразование данных в массив ключ + значение по у
    pub fn y(&self) -> Vec<(f64, f64)> {
        self.data.iter().map(|v| (v.key, v.value_y)).collect()
    }
}
impl DataArray<TrimDraughtData> {
    /// Преобразовает и возвращает данные
    pub fn data(mut self) -> Vec<(f64, Vec<(f64, f64)>)> {
        let mut vec: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();
        self.data.sort_by(|a, b| {
            a.trim
                .partial_cmp(&b.trim)
                .expect("DataArray<TrimVolumeData> data sort error!")
        });
        self.data.into_iter().for_each(|v| {
            if vec.last().is_none() || vec.last().unwrap().0 != v.trim {
                vec.push((v.trim, vec![(v.draught, v.value)]));
            } else {
                vec.last_mut().unwrap().1.push((v.draught, v.value));
            }
        });
        vec
    }
}
//
impl DataArray<TrimVolumeData> {
    /// Преобразовает и возвращает данные
    pub fn data(mut self) -> Vec<(f64, Vec<(f64, f64)>)> {
        let mut vec: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();
        self.data.sort_by(|a, b| {
            a.trim
                .partial_cmp(&b.trim)
                .expect("DataArray<TrimVolumeData> data sort error!")
        });
        self.data.into_iter().for_each(|v| {
            if vec.last().is_none() || vec.last().unwrap().0 != v.trim {
                vec.push((v.trim, vec![(v.volume, v.value)]));
            } else {
                vec.last_mut().unwrap().1.push((v.volume, v.value));
            }
        });
        vec
    }
}
//
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub point_x: Option<f64>,
    pub point_y: Option<f64>,
    pub point_z: Option<f64>,
}
//
impl DataArray<Point> {
    /// Преобразовает и возвращает данные
    pub fn data(self) -> Vec<Position> {
        self.data
            .into_iter()
            .filter(|v| v.point_x.is_some() && v.point_y.is_some() && v.point_z.is_some())
            .map(|v| Position::new(v.point_x.unwrap(), v.point_y.unwrap(), v.point_z.unwrap()))
            .collect()
    }
}
//
pub type PointDataArray = DataArray<Point>;
