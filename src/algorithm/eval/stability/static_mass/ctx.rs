use crate::algorithm::entities::{Moment, ship_model::{BulkData, GaseousData, LiquidData}};

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct StaticMassStabCtx {
    // Суммарная масса корпуса, обледенения с намоканием и грузов за вычетом смещяемых и насыпных грузов
    pub mass_const: f64,
    // Сумарный момент за вычетом смещяемых и насыпных груов
    pub moment_const: Moment,
    /// Груз, для которого центр массы и распределение зависит от 
    /// объема и/или положения корпуса, считается в модели
    pub bulk: Vec<BulkData>,
    pub liquid: Vec<LiquidData>,
    /// Для газообразного груза в модели считается распределение
    /// по шпациям
    pub gaseous: Vec<GaseousData>,
    /// Положение зерновых перегородок, координата по х
    pub grain_bulkhead: Vec<f64>,
}
