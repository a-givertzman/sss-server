//! Промежуточные структуры для serde_json для парсинга данных
//! Координаты отметок заглубления на корпусе судна
//! относительно центра корпуса судна
use std::collections::HashMap;
use crate::algorithm::entities::Position;
use super::{DataArray, PointData};
/// Координаты отметок заглубления на корпусе судна
/// относительно центра
pub type DraftMarkDataArray = DataArray<PointData>;
//
impl DraftMarkDataArray {
    /// Преобразование данных в массив
    pub fn draft_data(&self) -> Vec<DraftMarkParsedData> {
        let mut map: HashMap<i32, (String, Vec<Position>)> = HashMap::new();
        self.data.iter().for_each(|v| {
            if let Some((_, vector)) = map.get_mut(&v.criterion_id) {
                vector.push(Position::new(v.x, v.y, v.z));
            } else {
                map.insert(v.criterion_id, (v.name.clone(), vec![Position::new(v.x, v.y, v.z)]));
            }
        });
        map.into_iter()
            .map(|v| DraftMarkParsedData {
                criterion_id: v.0,
                name: v.1.0,
                data: v.1.1,
            })
            .collect()
    }
}
//
#[derive(Debug, Clone, PartialEq)]
pub struct DraftMarkParsedData {
    /// id
    pub criterion_id: i32,
    /// Имя
    pub name: String,
    /// Координаты относительно центра корпуса судна, м
    pub data: Vec<Position>,
}
//
impl std::fmt::Display for DraftMarkParsedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DraftMarkParsedData(criterion_id:{} name:{} data:{:?})",
            self.criterion_id, self.name, self.data
        )
    }
}
