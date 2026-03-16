//! Промежуточные структуры для serde_json для парсинга пантокаренов  
use super::DataArray;
use serde::{Deserialize, Serialize};
/// Промежуточные структуры для serde_json для парсинга данных  
/// плечей устойчивости формы (пантокарены)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PantocarenData {
    /// Дифферент
    pub trim: f64,
    /// Осадка при плотности воды 1.
    pub draught: f64,
    /// Крен, градус
    pub roll: f64,
    /// Плечо устойчивости, м
    pub moment: f64,
}
//
impl std::fmt::Display for PantocarenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PantocarenData(trim:{}, draught:{}, roll:{}, moment:{} )",
            self.trim, self.draught, self.roll, self.moment,
        )
    }
}
//
pub type PantocarenDataArray = DataArray<PantocarenData>;
pub type PantocarenVec = Vec<(f64, Vec<(f64, Vec<(f64, f64)>)>)>;
//
impl PantocarenDataArray {
    /// Преобразовает и возвращает данные
    /// trim | draught | roll | moment
    pub fn data(self) -> PantocarenVec {
        PantocarenDataArray::sort_by_trim(
            self.data
                .into_iter()
                .map(|v| (v.trim, v.draught, v.roll, v.moment))
                .collect(),
        )
    }
    /// Преобразовает и возвращает данные разбитые на вектора (крен, момент) по дифференту
    fn sort_by_trim(
        mut data: Vec<(f64, f64, f64, f64)>,
    ) -> PantocarenVec {
        let mut tmp: Vec<(f64, Vec<(f64, f64, f64)>)> = Vec::new();
        let mut res: PantocarenVec = Vec::new();
        data.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .expect("PantocarenDataArray data sort by trim error!")
        });
        data.into_iter().for_each(|v| {
            if tmp.last().is_none() || tmp.last().unwrap().0 != v.0 {
                tmp.push((v.0, vec![(v.1, v.2, v.3)]));
            } else {
                tmp.last_mut().unwrap().1.push((v.1, v.2, v.3));
            }
        });
        tmp.into_iter().for_each(|(trim, v)| {
            res.push((trim, PantocarenDataArray::sort_by_draught(v)));
        });
        res
    }
    /// Преобразовает и возвращает данные разбитые на вектора (крен, момент) по дифференту
    fn sort_by_draught(mut data: Vec<(f64, f64, f64)>) -> Vec<(f64, Vec<(f64, f64)>)> {
        let mut res: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();
        data.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .expect("PantocarenDataArray data sort by draught error!")
        });
        data.into_iter().for_each(|v| {
            if res.last().is_none() || res.last().unwrap().0 != v.0 {
                res.push((v.0, vec![(v.1, v.2)]));
            } else {
                res.last_mut().unwrap().1.push((v.1, v.2));
            }
        });
        res
    }
}
