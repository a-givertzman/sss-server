///
/// Координаты угловых точек переборок отсеков
#[derive(Debug, Clone)]
pub struct CompartmentCornerPoints {
    pub coordinates: Vec<Vec<(f64,f64,f64)>>, // (X, Z, Y)
}
//
//
impl CompartmentCornerPoints {
    ///
    /// Новый экземпляр [CompartmentCornerPoints]
    pub fn new() -> Self {
        Self { coordinates: Vec::new() }
    }
}