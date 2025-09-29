use bincode::{Decode, Encode};
///
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct GaseousResult {
    /// ID груза
    pub cargo_id: usize, 
    /// ID помещения
    pub space_id: String, 
    /// Распределение массы по шпациям, (index, value)
    pub mass_values: Vec<(usize, f64)>,
}
