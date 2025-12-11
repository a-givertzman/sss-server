///
/// Координаты поверхности наружного корпуса
#[derive(Debug, Clone)]
pub struct SurfaceOuterBody {
    pub coordinates: Vec<Vec<(f64,f64,f64)>>, // (X, Z, Y) кроме главных палуб
    pub main_deck: Vec<usize> // (X, Z, Y) позиции главной палубы
}
//
//
impl SurfaceOuterBody {
    ///
    /// Новый экземпляр [SurfaceOuterBody]
    pub fn new() -> Self {
        Self { 
            coordinates: Vec::new(),
            main_deck: Vec::new()
        }
    }
}