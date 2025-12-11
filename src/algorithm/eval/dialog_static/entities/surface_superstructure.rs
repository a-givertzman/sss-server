///
/// Координаты поверхности надстройки
#[derive(Debug, Clone)]
pub struct SurfaceSuperstructure {
    pub coordinates: Vec<Vec<(f64,f64,f64)>>, // (X, (Z,Y))
}
//
//
impl SurfaceSuperstructure {
    ///
    /// Новый экземпляр [SurfaceSuperstructure]
    pub fn new() -> Self{
        Self { coordinates: Vec::new() }
    }
}