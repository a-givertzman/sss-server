//! Промежуточные структуры для serde_json для парсинга данных объемного кренящего момента для зерна
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::algorithm::entities::data::DataArray;
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
