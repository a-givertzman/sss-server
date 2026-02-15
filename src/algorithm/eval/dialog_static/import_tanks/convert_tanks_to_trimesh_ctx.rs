use parry3d_f64::shape::TriMesh;
///
/// Результат преобразования координаты 3D модели 
/// в тип данных TriMesh
#[derive(Debug, Clone)]
pub struct ConvertTanksToTrimeshCtx {
    pub compartment_corner_points: Option<TriMesh>,
}