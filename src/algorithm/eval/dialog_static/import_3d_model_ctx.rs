use crate::algorithm::eval::entities::{
    diametrical_buttocks::DiametricalButtocks, 
    surface_outer_body::SurfaceOuterBody, 
    surface_superstructure::SurfaceSuperstructure
};
///
/// Координаты каждого блока 3D модели
/// из файла `DialogStatic`
#[derive(Debug, Clone)]
pub struct Import3DModelCtx {
    pub stern_block: DiametricalButtocks,
    pub nasal_block: DiametricalButtocks,
    pub surface_outer_body: SurfaceOuterBody,
    pub surface_superstructure: SurfaceSuperstructure,
}