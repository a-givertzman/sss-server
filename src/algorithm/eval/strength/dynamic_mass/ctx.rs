//! Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
use std::collections::HashMap;
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct DynamicMassCtx {
    /// Распределение массы по вектору разбиения
    pub mass_distr: Vec<f64>,
    /// Набор результатов расчетов - распределение массы по типам
    pub data: HashMap<String, Vec<f64>>,
}
//
impl DynamicMassCtx {
    /// Основной конструктор
    /// * data - Набор результатов расчетов - распределение массы по типам 
    /// * mass_distr - Распределение массы по вектору разбиения
    pub fn new(
        mass_distr: Vec<f64>,     
        data: HashMap<String, Vec<f64>>,
    ) -> Self {
        Self {
            mass_distr,            
            data,
        }
    }
}
