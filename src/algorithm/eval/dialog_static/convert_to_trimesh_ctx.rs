use parry3d_f64::shape::TriMesh;
///
/// Результат преобразования координаты 3D модели 
/// в тип данных TriMesh
#[derive(Debug, Clone)]
pub struct ConvertToTrimeshCtx {
    pub stern_block: Vec<TriMesh>,
    pub nasal_block: Vec<TriMesh>,
    pub surface_outer_body: Vec<TriMesh>,
    pub surface_superstructure: Vec<TriMesh>,
}