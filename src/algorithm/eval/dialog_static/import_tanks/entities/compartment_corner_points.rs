///
/// Координаты угловых точек переборок отсеков
#[derive(Debug, Clone)]
pub struct CompartmentCornerPoints {
    pub id: f64,
    pub coordinates_x1: (f64,Vec<(f64,f64)>), // (X, (Z, Y))
    pub coordinates_x2: (f64,Vec<(f64,f64)>), // (X, (Z, Y))
}
//
//
impl CompartmentCornerPoints {
    ///
    /// Новый экземпляр [CompartmentCornerPoints]
    pub fn new() -> Self {
        Self {
            id: 0.0, 
            coordinates_x1: (0.0, Vec::new()),
            coordinates_x2: (0.0, Vec::new())
        }
    }
}