use indexmap::IndexMap;
///
/// Store coordinates of surface superstructure
pub struct SurfaceSuperStructure {
    pub coordinates: IndexMap<f64,(f64,f64)>, // (X, (Z, Y))
}