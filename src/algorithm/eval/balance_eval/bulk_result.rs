use bincode::{Decode, Encode};
use crate::algorithm::entities::{Position, data::loads::AssignmentType};
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct BulkResult {
    /// ID груза
    pub cargo_id: usize, 
    /// ID помещения
    pub space_id: String, 
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
  //  /// смещение центра массы
 //   pub mass_shift: Position,
    /// Объемный кренящий момент
    pub moment: f64,
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,    
}
///
impl BulkResult {
    ///
    pub fn new(
        cargo_id: usize, 
        space_id: String,
        assigment_type: AssignmentType,
        moment: f64,
        mass_values: Vec<f64>,
    ) -> Self {
        Self {
            cargo_id,
            space_id,
            assigment_type,
            moment,
            mass_values,
        }
    }
}
