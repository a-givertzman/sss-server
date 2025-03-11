//! Промежуточные структуры для serde_json для парсинга данных
//! Отстояние центра величины погруженной части судна в
//! зависимости от дифферента и объемного водоизмещения
use serde::{Deserialize, Serialize};
use crate::algorithm::entities::{data::DataArray, Position};
/// Отстояние центра величины погруженной части судна
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CenterDraughtShift {
    /// Trim, m
    pub trim: f64,
    /// V, м3
    pub volume: f64,
    /// Xc, м
    pub value_x: f64,
    /// Yc, м
    pub value_y: f64,
    /// Zc, м
    pub value_z: f64,
}
//
impl std::fmt::Display for CenterDraughtShift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CenterDraughtShift(trim:{}, volume:{:?} value_x:{}, value_y:{}, value_z:{})",
            self.trim, self.volume, self.value_x, self.value_y, self.value_z,
        )
    }
}
//
pub type CenterDraughtShiftArray = DataArray<CenterDraughtShift>;
//
impl CenterDraughtShiftArray {
    /// Преобразовает и возвращает данные
    pub fn data(mut self) -> Vec<(f64, Vec<(f64, Position)>)> {
        let mut vec: Vec<(f64, Vec<(f64, Position)>)> = Vec::new();
        self.data.sort_by(|a, b| {
            a.trim
                .partial_cmp(&b.trim)
                .expect("CenterDraughtShiftArray data sort error!")
        });
        self.data.into_iter().for_each(|v| {
            if vec.last().is_none() || vec.last().unwrap().0 != v.trim {
                vec.push((v.trim, vec![(v.volume, Position::new(v.value_x, v.value_y, v.value_z))]));
            } else {
                vec.last_mut().unwrap().1.push((v.volume, Position::new(v.value_x, v.value_y, v.value_z)));
            }
        });
        vec.iter_mut().for_each(|(_, v)| v.sort_by(|&a, &b| a.0.partial_cmp(&b.0).unwrap()));
        vec
    }    
}
