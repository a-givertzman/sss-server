use bincode::{Decode, Encode};
///
/// TODO: Type doc here
#[derive(Debug, Clone, Decode, Encode)]
pub struct LiquidResult {
    /// ID помещения
    pub space_id: String, 
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
        long_moment_of_inertia: f64,
        trans_moment_of_inertia: f64,
    ) -> Self {
        Self {
            space_id,
            long_moment_of_inertia,
            trans_moment_of_inertia,
        }
    }
}
