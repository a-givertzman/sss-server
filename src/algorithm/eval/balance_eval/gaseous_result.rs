use bincode::{Decode, Encode};

use crate::algorithm::entities::data::loads::AssignmentType;
///
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct GaseousResult {
    /// ID груза
    pub cargo_id: usize, 
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
    pub fn new(cargo_id: usize, space_id: String, assigment_type: AssignmentType, mass_values: Vec<f64>) -> Self {
        Self {
            cargo_id,
            space_id,
            assigment_type,
            mass_values,
        }
    }
}
