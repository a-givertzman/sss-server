use bincode::{Decode, Encode};
///
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct GaseousResult {
    /// ID груза
    pub cargo_id: usize, 
    /// ID помещения
    pub space_id: String, 
    /// Распределение массы по шпациям
    pub mass_values: Vec<f64>,
}
///
impl GaseousResult {
    ///
    pub fn new(cargo_id: usize, space_id: String, mass_values: Vec<f64>) -> Self {
        Self {
            cargo_id,
            space_id,
            mass_values,
        }
    }
}
