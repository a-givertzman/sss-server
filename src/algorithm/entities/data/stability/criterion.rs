use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::DataArray;
// Структура для парсинга данных критериев
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CriterionRelation {
    pub id: i32,
    pub relation: Option<String>,
}
//
impl std::fmt::Display for CriterionRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CriterionRelation(id:{}, relation:{:?}", self.id, self.relation)
    }
}
//
pub type CriterionRelationArray = DataArray<CriterionRelation>;
//
impl CriterionRelationArray {
    /// Преобразование данных в массив ключ + значение
    pub fn data(&self) -> HashMap<i32, String> {
        self.data
            .iter()
            .filter(|v| v.relation.is_some() )
            .map(|v| {
                (v.id, v.relation.clone().unwrap())
            })
            .collect()
    }
}
