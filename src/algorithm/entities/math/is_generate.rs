use nalgebra::{Const, OPoint};
///
/// Проверка на вырожденность треугольника
/// - `vertices` - набор вершин треугольника
/// - `tri` - индексы вершин треугольника
pub fn is_degenerate(
    vertices: &[OPoint<f64, Const<3>>],
    tri: [u32; 3],
) -> bool {
    let a = vertices[tri[0] as usize];
    let b = vertices[tri[1] as usize];
    let c = vertices[tri[2] as usize];
    let ab = b - a;
    let ac = c - a;
    ab.cross(&ac).norm_squared() < 1e-12
}