use nalgebra::{Const, OPoint};
use parry3d_f64::math::Point;
///
/// Закрытие сэмпла веером
pub struct BuildWall;
//
//
impl BuildWall {
    ///
    /// Закрытие сэмпла веером
    /// - `reverse` - ориентация нормалей
    /// - `x2` - индекс начала вершин закрываемого сэмпла
    /// - `vertices` - вершины 3D модели
    /// - `indices` - массив индексов треугольников фигуры
    /// - `points` - вершины закрываемого сэмпла
    /// - `n` - кол-во точек закрываемого сэмпла
    pub fn eval(
        reverse: bool, 
        x2: u32, 
        indices: &mut Vec<[u32; 3]>, 
        vertices: &mut Vec<OPoint<f64, Const<3>>>,
        points: &Vec<OPoint<f64, Const<3>>>, 
        n: usize,
    ) {
        let centroid = Self::calculate_centroid(points);
        let center_idx = vertices.len() as u32;
        vertices.push(centroid);
        for i in 0..n {
            let next = (i + 1) % n;
            if reverse {
                indices.push([
                    center_idx,
                    x2 + next as u32,
                    x2 + i as u32,
                ]);
            } else {
                indices.push([
                    center_idx,
                    x2 + i as u32,
                    x2 + next as u32,
                ]);
            }
        }
    }   
    ///
    /// Вычисляется центроида сэмпла
    /// - `points` - точки у которых надо найти центроид
    fn calculate_centroid(
        points: &[Point<f64>]
    ) -> Point<f64> {
        let sum = points.iter().fold(Point::new(0.0, 0.0, 0.0), |acc, p| {
            Point::new(acc.x + p.x, acc.y + p.y, acc.z + p.z)
        });
        let count = points.len() as f64;
        Point::new(sum.x / count, sum.y / count, sum.z / count)
    }
}