use bincode::{Decode, Encode};

use crate::algorithm::entities::data::loads::AssignmentType;
///
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct GaseousResult {
    /// ID помещения
    pub space_id: String,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,
}
///
impl GaseousResult {
    ///
    pub fn new(
        space_id: String, 
        assigment_type: AssignmentType, 
        mass_values: Vec<f64>
    ) -> Self {
        Self {
            space_id,
            assigment_type,
            mass_values,
        }
    }
}
