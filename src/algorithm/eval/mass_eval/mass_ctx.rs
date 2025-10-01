//! Нагрузка на корпус судна: конструкции, груз, экипаж и т.п.
use std::collections::HashMap;
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct MassCtx {
    /// Набор результатов расчетов
    data: HashMap<String, Vec<f64>>,
    /// Распределение массы по вектору разбиения
    mass_values: Vec<f64>,
}
//
impl MassCtx {
    /// Основной конструктор
    /// * data - Набор результатов расчетов 
    /// * mass_values - Распределение массы по вектору разбиения
    pub fn new(
        data: HashMap<String, Vec<f64>>,
        mass_values: Vec<f64>
    ) -> Self {
        Self {
            data,
            mass_values,
        }
    }
}
