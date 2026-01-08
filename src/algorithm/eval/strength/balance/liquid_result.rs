use bincode::{Decode, Encode};
use crate::algorithm::entities::data::loads::{AssignmentType, LiquidCargoType};
///
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct LiquidResult {
    /// ID помещения
    pub space_id: String, 
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// Тип жидкого груза
    pub cargo_type: LiquidCargoType,
 //   /// смещение центра массы
 //   pub mass_shift: Position,
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,
}
///
impl LiquidResult {
    ///
    pub fn new(
        space_id: String,
        assigment_type: AssignmentType,
        cargo_type: LiquidCargoType,
  //      mass_shift: Position,
        mass_values: Vec<f64>,
    ) -> Self {
        Self {
            space_id,
            assigment_type,
            cargo_type,
     //       mass_shift,
            mass_values,
        }
    }
}
