//! Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
use std::collections::HashMap;
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct DynamicMassCtx {
    /// Распределение массы по вектору разбиения
    pub values: Vec<f64>,
    /// Набор результатов расчетов - распределение массы по типам
    pub data: HashMap<String, Vec<f64>>,
}
//
impl DynamicMassCtx {
    /// Основной конструктор
    /// * data - Набор результатов расчетов - распределение массы по типам 
    /// * values - Распределение массы по вектору разбиения
    pub fn new(
        values: Vec<f64>,     
        data: HashMap<String, Vec<f64>>,
    ) -> Self {
        Self {
            values,            
            data,
        }
    }
}
