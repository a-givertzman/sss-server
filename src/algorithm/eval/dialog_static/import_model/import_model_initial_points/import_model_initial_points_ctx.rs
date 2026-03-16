///
/// 
#[derive(Debug, Clone)]
pub struct SurfaceOuterBody {
    pub coordinates: Vec<Vec<(f64, f64, f64)>>,
    pub main_deck: Vec<usize>,
}
///
/// Координаты 3D модели
/// из файла `DialogStatic`
/// - `stern_block` - блок координат кормового диаметрального баттокса
/// - `bow_block` - блок координат носового диаметрального баттокса
/// - `surface_outer_body` - блок координат поверхности наружного корпуса
/// - `surface_superstructure` - блок координат поверхности надстройки
#[derive(Debug, Clone)]
pub struct ImportModelInitialPointsCtx {
    pub stern_block: Vec<(f64, f64, f64)>,
    pub bow_block: Vec<(f64, f64, f64)>,
    pub surface_outer_body: SurfaceOuterBody,
    pub surface_superstructure: Vec<Vec<(f64, f64, f64)>>,
}
