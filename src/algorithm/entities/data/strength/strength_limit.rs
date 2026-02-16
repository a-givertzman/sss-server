//! Промежуточные структуры для serde_json для парсинга данных
//! расчета прочности
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::algorithm::entities::data::DataArray;
/// Данные расчета прочности
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrengthLimitData {
    pub frame_x: f64,
    pub value: f64,
    pub limit_type: String, //'low', 'high'
    pub force_type: String, //'shear_force', 'bending_moment'
}
//
impl StrengthLimitData {
    pub fn new(
        frame_x: f64,
        value: f64,
        limit_type: &str, //'low', 'high'
        force_type: &str,
    ) -> Self {
        Self {
            frame_x,
            value,
            limit_type: limit_type.to_owned(),
            force_type: force_type.to_owned(),
        }
    }
}
//
impl std::fmt::Display for StrengthLimitData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StrengthLimitData(frame_x:{}, value:{}, limit_type:{}, force_type:{} )",
            self.frame_x, self.value, self.limit_type, self.force_type,
        )
    }
}
pub type StrengthLimitDataArray = DataArray<StrengthLimitData>;
//
impl StrengthLimitDataArray {
    // ( bm_min, bm_max, sf_min, sf_max)
    pub fn data(
        &self,
    ) -> (
        Vec<(f64, f64)>,
        Vec<(f64, f64)>,
        Vec<(f64, f64)>,
        Vec<(f64, f64)>,
    ) {
        convert(&self.data.iter().collect::<Vec<_>>())
    }
}
//
impl From<Vec<(f64, f64, &str, &str)>> for StrengthLimitDataArray {
    fn from(src: Vec<(f64, f64, &str, &str)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(frame_x,
                    value,
                    limit_type, 
                    force_type,
                )| StrengthLimitData::new(
                    frame_x,
                    value,
                    limit_type,
                    force_type,
                ))
                .collect(),
            error: HashMap::new(),
        }
    }
}
// (sf_min, sf_max, bm_min, bm_max): Vec<(x, value)>
fn convert(
    data: &Vec<&StrengthLimitData>,
) -> (
    Vec<(f64, f64)>,
    Vec<(f64, f64)>,
    Vec<(f64, f64)>,
    Vec<(f64, f64)>,
) {
    let mut sf_min = Vec::new();
    let mut sf_max = Vec::new();
    let mut bm_min = Vec::new();
    let mut bm_max = Vec::new();
    for v in data {
        if v.force_type.contains("shear_force") {
            if v.limit_type.contains("low") {
                sf_min.push((v.frame_x, v.value));
            } else {
                // 'high'
                sf_max.push((v.frame_x, v.value));
            }
        } else {
            // 'bending_moment'
            if v.limit_type.contains("low") {
                bm_min.push((v.frame_x, v.value));
            } else {
                // 'high'
                bm_max.push((v.frame_x, v.value));
            }
        }
    }
    (sf_min, sf_max, bm_min, bm_max)
}
