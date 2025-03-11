use std::collections::HashMap;
use crate::data::structs::{DataArray, Pair, PointData, Quadruple, TrimDraughtData, TrimVolumeData, Triple};
//
impl From<Vec<(f64, f64)>> for DataArray<Pair> {
    fn from(src: Vec<(f64, f64)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(key, value)| Pair { key, value })
                .collect(),
            error: HashMap::new(),
        }
    }
}
//
impl From<Vec<(f64, f64, f64)>> for DataArray<Triple> {
    fn from(src: Vec<(f64, f64, f64)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(key, value_x, value_y)| Triple {
                    key,
                    value_x,
                    value_y,
                })
                .collect(),
            error: HashMap::new(),
        }
    }
}
//
impl From<Vec<(f64, f64, f64, f64)>> for DataArray<Quadruple> {
    fn from(src: Vec<(f64, f64, f64, f64)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(key, value_x, value_y, value_z)| Quadruple {
                    key,
                    value_x,
                    value_y,
                    value_z,
                })
                .collect(),
            error: HashMap::new(),
        }
    }
}
//
impl From<Vec<(i32, f64, f64, f64)>> for DataArray<PointData> {
    fn from(src: Vec<(i32, f64, f64, f64)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(criterion_id, x, y, z)| PointData::new(
                    criterion_id,
                    "_".to_string(),
                    x,
                    y,
                    z,
                ))
                .collect(),
            error: HashMap::new(),
        }
    }
}
//
impl From<Vec<(f64, f64, f64)>> for DataArray<TrimVolumeData> {
    fn from(src: Vec<(f64, f64, f64)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(trim, volume, value)| TrimVolumeData {
                    trim,
                    volume,
                    value,
                })
                .collect(),
            error: HashMap::new(),
        }
    }
}
//
impl From<Vec<(f64, f64)>> for DataArray<TrimVolumeData> {
    fn from(src: Vec<(f64, f64)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(volume, value)| TrimVolumeData {
                    trim: 0.,
                    volume,
                    value,
                })
                .collect(),
            error: HashMap::new(),
        }
    }
}
//
impl From<Vec<(f64, f64, f64)>> for DataArray<TrimDraughtData> {
    fn from(src: Vec<(f64, f64, f64)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(trim, draught, value)| TrimDraughtData {
                    trim,
                    draught,
                    value,
                })
                .collect(),
            error: HashMap::new(),
        }
    }
}
//
impl From<Vec<(f64, f64)>> for DataArray<TrimDraughtData> {
    fn from(src: Vec<(f64, f64)>) -> Self {
        Self {
            data: src
                .into_iter()
                .map(|(draught, value)| TrimDraughtData {
                    trim: 0.,
                    draught,
                    value,
                })
                .collect(),
            error: HashMap::new(),
        }
    }
}
