use crate::algorithm::entities::Position;

///
#[derive(Debug, Clone)]
pub struct LiquidData {
    /// ID помещения
    pub space_id: usize, 
    pub mass: f64, 
    pub volume: f64,
}
///
#[derive(Debug, Clone)]
pub struct GaseousData {
    /// ID помещения
    pub space_id: usize, 
    pub mass: f64, 
}
///
#[derive(Debug, Clone)]
pub struct GaseousData {
    /// ID помещения
    pub space_id: usize, 
    pub mass: f64, 
}
///
#[derive(Debug, Clone)]
pub struct MassCtx {
    pub hull: f64, 
    pub hull_shift: Position,
    pub unit: f64, 
    pub unit_shift: Position,
    pub liquid: Vec<(usize, f64, )>,

    /// Суммарная масса балласта
    pub ballast: f64,
    /// Суммарная масса запасов
    pub stores: f64,
    /// Суммарная масса обледенения
    pub icing: f64,
    /// Суммарная масса намокания
    pub wetting: f64,
    /// Суммарная масса груза
    pub cargo: f64,
    /// Суммарная масса зерновых перегородок
    pub bulkhead: f64,
    /// Суммарная масса корпуса
    pub lightship: f64,
    /// Суммарная масса
    pub mass_sum: f64,
    /// Смещение центра площади
    pub mass_shift: Position,
    /// Распределение массы по вектору разбиения
    pub mass_values: Vec<f64>,
}
