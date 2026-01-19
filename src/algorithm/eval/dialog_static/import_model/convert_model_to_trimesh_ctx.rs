use parry3d_f64::shape::TriMesh;
///
/// Результат преобразования координаты 3D модели 
/// в тип данных TriMesh
#[derive(Debug, Clone)]
pub struct ConvertModelToTrimeshCtx {
    pub stern_block: Option<TriMesh>,
    pub nasal_block: Option<TriMesh>,
    pub surface_outer_body: Option<TriMesh>,
    pub surface_superstructure: Option<TriMesh>,
}
