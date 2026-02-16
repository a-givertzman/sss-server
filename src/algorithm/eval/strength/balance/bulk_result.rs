use bincode::{Decode, Encode};
use crate::algorithm::entities::data::loads::AssignmentType;
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct BulkResult {
    /// ID помещения
    pub code: String, 
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,    
}
///
impl BulkResult {
    ///
    pub fn new(
        code: String,
        assigment_type: AssignmentType,
        mass_values: Vec<f64>,
    ) -> Self {
        Self {
            code,
            assigment_type,
            mass_values,
        }
    }
}
