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
 //   /// смещение центра массы
 //   pub mass_shift: Position,
    /// продольный момент свободной поверхности жидкости
    pub long_moment_of_inertia: f64,
    /// поперечный момент свободной поверхности жидкости
    pub trans_moment_of_inertia: f64,
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,
}
///
impl LiquidResult {
    ///
    pub fn new(
        cargo_id: usize,
        space_id: String,
        assigment_type: AssignmentType,
        cargo_type: LiquidCargoType,
  //      mass_shift: Position,
        long_moment_of_inertia: f64,
        trans_moment_of_inertia: f64,
        mass_values: Vec<f64>,
    ) -> Self {
        Self {
            cargo_id,
            space_id,
            assigment_type,
            cargo_type,
     //       mass_shift,
            long_moment_of_inertia,
            trans_moment_of_inertia,
            mass_values,
        }
    }
}
