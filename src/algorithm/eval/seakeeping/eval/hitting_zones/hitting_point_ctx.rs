use std::collections::HashMap;
#[derive(Debug, Clone, Default)]
///
/// Результаты расчета пересечения с зонами резонанса
pub struct HittingZonesCtx {
    /// Массив попаданий в зоны
    pub hitting_zones: HashMap<String,bool> 
}
