use bincode::{Decode, Encode};
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct BulkResult {
    /// ID помещения
    pub space_id: String, 
    /// Объемный кренящий момент
    pub moment: f64,
}
///
impl BulkResult {
    ///
    pub fn new(
        space_id: String,
        moment: f64,
    ) -> Self {
        Self {
            space_id,
            moment,
        }
    }
}
