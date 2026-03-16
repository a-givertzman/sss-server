use bincode::{Decode, Encode};
use crate::algorithm::entities::Position;

///
/// Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct BulkResult {
    /// ID груза
    pub cargo_id: usize,
    /// ID помещения
    pub space_id: usize,
    /// смещение центра массы
    pub mass_shift: Position,
    /// Распределение массы по шпациям, (index, value)
    pub mass_values: Vec<(usize, f64)>,
    /// Объемный кренящий момент
    pub moment: f64,
}
