use bincode::{Decode, Encode};
use crate::algorithm::entities::{data::loads::{AssignmentType, LiquidCargoType}, Position};
///
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct LiquidResult {
    /// ID груза
    pub cargo_id: usize, 
    /// ID помещения
    pub space_id: String, 
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// Тип жидкого груза
    pub cargo_type: LiquidCargoType,
    /// смещение центра массы
    pub mass_shift: Position,
    /// продольный момент свободной поверхности жидкости
    pub long_moment_of_inertia: f64,
    /// поперечный момент свободной поверхности жидкости
    pub trans_moment_of_inertia: f64,
    /// Распределение массы по шпациям, (index, value)
    pub mass_values: Vec<(usize, f64)>,
}
