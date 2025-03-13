use crate::algorithm::entities::data::{loads::*, IcingArray, Ship, ShipParametersArray, Voyage};

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct InitialCtx {
    pub ship_id: usize,
    /// разбиение на шпации - фреймы
    pub bounds: Option<Vec<(f64, f64)>>,
    /// Текстовые данные по судну
    pub ship: Option<Ship>,
    /// Численные данные по судну
    pub ship_parameters: Option<ShipParametersArray>,
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
    pub fn new(ship_id: usize) -> Self {
        Self {
            ship_id,
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
            ship_id: 0,
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
