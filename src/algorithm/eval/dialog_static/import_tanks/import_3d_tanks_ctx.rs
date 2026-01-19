use crate::algorithm::eval::entities::compartment_corner_points::CompartmentCornerPoints;
///
/// Координаты каждого блока 3D отсека
/// из файла `DialogStatic`
#[derive(Debug, Clone)]
pub struct Import3DTanksCtx {
    pub compartment_corner_points: CompartmentCornerPoints,
}