//! Расчет уровня заглубления для координат отметок заглубления на корпусе судна
//! Результат пишется в параметры
pub mod draft_mark_ctx;
pub mod draft_mark_eval;

/// Результат расчета уровня заглубления
#[derive(Clone, Debug, PartialEq)]
pub struct DraftMarkResult {
    /// id критерия
    pub criterion_id: i32,
    /// Имя
    pub name: String,
    /// Координаты
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
}
//
impl DraftMarkResult {
    /// Основной конструктор
    pub fn new(criterion_id: i32, name: String, x: f64, y: f64, z: Option<f64>) -> Self {
        Self {
            criterion_id,
            name,
            x,
            y,
            z,
        }
    }
}
