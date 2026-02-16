use bincode::{Decode, Encode};
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct BulkResult {
    /// ID помещения
    pub code: String, 
    /// Объемный кренящий момент
    pub moment: f64,
}
///
impl BulkResult {
    ///
    pub fn new(
        code: String,
        moment: f64,
    ) -> Self {
        Self {
            code,
            moment,
        }
    }
}
