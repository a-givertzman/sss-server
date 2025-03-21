use crate::algorithm::entities::{data::loads::*, *};
///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct LoadsCtx {
    /// Постоянная нагрузка на судно
    pub load_constant: Vec<LoadConstantData>,
    /// Смещение центра масс постоянной нагрузки на судно
    pub shift_const: Position,
    /// Переменная нагрузка на судно
    pub bulk: Vec<LoadBulkData>,
    pub liquid: Vec<LoadLiquidData>,
    pub unit: Vec<LoadUnitData>,
    pub gaseous: Vec<LoadGaseousData>
}
