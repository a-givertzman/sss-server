use bincode::{Decode, Encode};

///
/// Груз, для которого центр массы и распределение зависит от 
/// объема и/или положения корпуса, считается в модели
#[derive(Debug, Clone, Decode, Encode)]
pub struct BulkData {
    pub cargo_id: usize, // ID груза
    pub space_id: String, // ID помещения
    pub mass: f64,
    pub volume: f64,
}
