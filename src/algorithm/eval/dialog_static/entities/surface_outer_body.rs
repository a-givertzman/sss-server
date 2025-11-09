use indexmap::IndexMap;
use multimap::MultiMap;
///
/// Координаты поверхности наружного корпуса
#[derive(Debug, Clone)]
pub struct SurfaceOuterBody {
    pub coordinates: Vec<Vec<(f64,f64,f64)>>, // (X, (Z,Y))
}
//
//
impl SurfaceOuterBody {
    ///
    /// Новый экземпляр [SurfaceOuterBody]
    pub fn new() -> Self{
        Self { coordinates: Vec::new() }
    }
}