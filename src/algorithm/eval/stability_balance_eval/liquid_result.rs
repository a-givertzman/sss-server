use bincode::{Decode, Encode};

use crate::algorithm::entities::data::loads::AssignmentType;
///
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct LiquidResult {
    /// ID помещения
    pub space_id: String, 
    /// Тип назначения груза
    pub assigment_type: AssignmentType,    
    /// продольный момент свободной поверхности жидкости
    pub long_moment_of_inertia: f64,
    /// поперечный момент свободной поверхности жидкости
    pub trans_moment_of_inertia: f64,
}
///
impl LiquidResult {
    ///
    pub fn new(
        space_id: String,
        assigment_type: AssignmentType,
        long_moment_of_inertia: f64,
        trans_moment_of_inertia: f64,
    ) -> Self {
        Self {
            space_id,
            assigment_type,
            long_moment_of_inertia,
            trans_moment_of_inertia,
        }
    }
}
