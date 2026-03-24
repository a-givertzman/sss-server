use parry3d_f64::shape::TriMesh;
///
/// Результат преобразования координат
/// в тип данных TriMesh
#[derive(Debug, Clone)]
pub struct ConvertToTrimeshCtx {
    pub result: Option<TriMesh>,
}