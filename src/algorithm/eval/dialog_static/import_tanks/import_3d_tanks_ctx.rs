use crate::algorithm::eval::entities::compartment_corner_points::CompartmentCornerPoints;
///
/// Координаты каждого блока 3D отсека
/// из файла `DialogStatic`
/// - `compartment_corner_points` - набор угловых точек отсеков
/// - `compartment_id_to_reverse` - id отсеков которые нужно отзеркалить по y
#[derive(Debug, Clone)]
pub struct Import3DTanksCtx {
    pub compartment_corner_points: Vec<CompartmentCornerPoints>,
    pub compartment_id_to_reverse: Vec<f64>,
}