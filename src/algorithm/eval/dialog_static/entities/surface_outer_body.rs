use indexmap::IndexMap;
///
/// Координаты поверхности наружного корпуса
#[derive(Debug, Clone)]
pub struct SurfaceOuterBody {
    pub coordinates: IndexMap<String,Vec<(f64,f64)>>, // (X, (Z,Y))
}
//
//
impl SurfaceOuterBody {
    ///
    /// Новый экземпляр [SurfaceOuterBody]
    pub fn new() -> Self{
        Self { coordinates: IndexMap::new() }
    }
}