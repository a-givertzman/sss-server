use std::collections::HashMap;
use crate::algorithm::entities::Bounds;
use crate::algorithm::entities::data::{loads::*, IcingArray, Ship, Voyage};

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct InitialCtx {
    pub ship_id: String,
    pub project_id: String,
    /// разбиение на шпации - фреймы
    pub bounds: Option<Bounds>,
    /// Текстовые данные по судну
    pub ship: Option<Ship>,
    /// Численные данные по судну
    pub ship_parameters: Option<HashMap<String, f64>>,
    /// Данные по обстановке
    pub voyage: Option<Voyage>,
    /// Данные по обледенению
    pub icing: Option<IcingArray>,
    /// Постоянная нагрузка на судно
    pub load_constant: Option<LoadConstantArray>,
    /// Переменная нагрузка на судно
    pub bulk: Option<LoadBulkArray>,
    pub liquid: Option<LoadLiquidArray>,
    pub unit: Option<LoadUnitArray>,
    pub gaseous: Option<LoadGaseousArray>,
}
impl InitialCtx {
    ///
    /// Struct constructor
    /// - 'ship_id' - the identifier of the ship in the database
    pub fn new(ship_id: usize, project_id: &str) -> Self {
        Self {
            ship_id: format!("{ship_id}"),
            project_id: project_id.to_owned(),
            ..Self::default()
        }
    }
}
//
//
impl Default for InitialCtx {
    ///
    /// Struct constructor
    /// - 'storage_initial_data' - [Storage] instance, where store initial data
    fn default() -> Self {
        Self {
            ship_id: "NUll".to_owned(),
            project_id: "NUll".to_owned(),
            bounds: None,
            ship: None,
            ship_parameters: None,
            voyage: None,
            icing: None,
            load_constant: None,
            bulk: None,
            liquid: None,
            unit: None,
            gaseous: None,
        }
    }
}
