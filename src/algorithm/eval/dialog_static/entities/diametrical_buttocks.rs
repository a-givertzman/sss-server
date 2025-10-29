///
/// Координаты диаметрального батокса
#[derive(Debug, Clone)]
pub struct DiametricalButtocks {
    pub coordinates: Vec<(f64,f64)>, // (Z,X)
}
//
//
impl DiametricalButtocks {
    ///
    /// Новый экземпляр [DiametricalButtocks]
    pub fn new() -> Self{
        Self { coordinates: Vec::new() }
    }
}