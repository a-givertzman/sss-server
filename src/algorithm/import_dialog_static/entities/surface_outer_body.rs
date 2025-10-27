use indexmap::IndexMap;

///
/// Store coordinates of surface outer body
pub struct SurfaceOuterBody {
    pub coordinates: IndexMap<f64,(f64,f64)>, // (X, (Z, Y))
}