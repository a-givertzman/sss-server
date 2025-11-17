use crate::algorithm::entities::{Position, ship_model::{BulkData, GaseousData, LiquidData}};

///
/// Общая структура для ввода данных. Содержит все данные
/// для расчетов.
#[derive(Debug, Clone)]
pub struct StaticMassCtx {
    /// Масса 
    pub mass_const: f64,
    pub mass_unit: f64,
    pub mass_gaseous: f64,
    /// Смещение центра масс
    pub shift_const: Position,
    pub shift_unit: Position,
    pub shift_gaseous: Position,
    /// Суммарное распределение статической массы и генеральных грузов: масса корпуса и механизмов, 
    /// намокания, обледенения и генерального грузов 
    /// отсутствует жидкий, газообразный и сыпучий груз, их распределение считается по модели отсеков
    pub distr_static: Vec<f64>,       
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
