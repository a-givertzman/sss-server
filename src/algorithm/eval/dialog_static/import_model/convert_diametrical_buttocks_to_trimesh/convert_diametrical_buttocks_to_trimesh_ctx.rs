use parry3d_f64::shape::TriMesh;
///
/// Результат преобразования координат
/// диаметральных баттоксов 3D модели 
/// в тип данных TriMesh
#[derive(Debug, Clone)]
pub struct ConvertDiametricalButtocksToTrimeshCtx {
    pub stern_buttocks: Option<TriMesh>,
    pub bow_buttocks: Option<TriMesh>, 
}