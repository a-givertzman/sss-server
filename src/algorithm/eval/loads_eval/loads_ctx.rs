use crate::algorithm::entities::{data::loads::*, *};
///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct LoadsCtx {
    /// Постоянная нагрузка на судно
    load_constant: Vec<LoadConstantData>,
    /// Смещение центра масс постоянной нагрузки на судно
    shift_const: Position,
    /// Переменная нагрузка на судно
    bulk: Vec<LoadBulkData>,
    liquid: Vec<LoadLiquidData>,
    unit: Vec<LoadUnitData>,
    gaseous: Vec<LoadGaseousData>
}
