use std::collections::HashMap;
use crate::data::structs::{ScrewData, ScrewDataArray};
//
impl From<Vec<(i32, f64, f64, f64, f64)>> for ScrewDataArray {
    fn from(src: Vec<(i32, f64, f64, f64, f64)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(criterion_id, x, y, z, d)| ScrewData {
                    criterion_id,
                    x,
                    y,
                    z,
                    d,
                })
                .collect(),
            error: HashMap::new(),
        }
    }
}
//
#[allow(dead_code)]
pub(crate) fn screw() -> ScrewDataArray {
    ScrewDataArray::from(vec![
        (146, 0., 3.575, 1.72, 2.4),
        (147, 0., -3.575, 1.72, 2.4),
    ])
}
