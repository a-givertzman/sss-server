use bincode::{Decode, Encode};

use crate::algorithm::entities::data::loads::AssignmentType;
///
/// Результат расчета по газообразным грузам
#[derive(Debug, Clone, Decode, Encode)]
pub struct GaseousResult {
    /// ID помещения
    pub code: String,
    /// Тип назначения груза
    pub assigment_type: AssignmentType,
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,
}
//
impl GaseousResult {
    //
    pub fn new(
        code: String, 
        assigment_type: AssignmentType, 
        mass_values: Vec<f64>
    ) -> Self {
        Self {
            code,
            assigment_type,
            mass_values,
        }
    }
}
