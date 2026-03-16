use nalgebra::{Const, OPoint};
use parry3d_f64::math::Point;

use crate::algorithm::entities::{is_generate::is_degenerate, resample_line::resample_line};
///
/// Трейт для преобразования в точки
pub trait IntoPoints {
    fn into_points(self) -> Vec<Point<f64>>;
}
//
// Реализация для слайса кортежей
impl IntoPoints for &[(f64, f64, f64)] {
    fn into_points(self) -> Vec<Point<f64>> {
        self.iter()
            .map(|&(x, y, z)| Point::new(x, z, y))
            .collect()
    }
}
//
// Реализация для постоянного X
impl IntoPoints for (f64, Vec<(f64, f64)>) {
    fn into_points(self) -> Vec<Point<f64>> {
        let (x, points) = self;
        points
            .into_iter()
            .map(|(z, y)| Point::new(x, -y, z))
            .collect()
    }
}
//
// Реализация для вектора кортежей
impl IntoPoints for Vec<(f64, f64, f64)> {
    fn into_points(self) -> Vec<Point<f64>> {
        self.into_iter()
            .map(|(x, y, z)| Point::new(x, z, y))
            .collect()
    }
}
///
/// Структура для манипуляций
/// и преобразований координат/точек
pub struct PointsManipulations;
//
//
impl PointsManipulations {
    ///
    /// Преобразование набор координат
    /// в вектор [Point]
    pub fn convert<T: IntoPoints>(input: T) -> Vec<Point<f64>> {
        input.into_points()
    }
    ///
    /// Разбиение сэмпла пополам
    /// с последующей интерполяцией
    /// по всей длине
    /// - `source` - сэмпл для разбиения
    /// - `target_points` - кол-во точек на сэмпл для интерполяции
    pub fn split_and_resample(
        source: &[Point<f64>],
        target_points: usize,
    ) -> (Vec<Point<f64>>, Vec<Point<f64>>) {
        let mut source_sort_by_y = source.to_vec();
        source_sort_by_y.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
        let y_mid = source_sort_by_y[source_sort_by_y.len() / 2].y;
        let mut upper: Vec<Point<f64>> =
            source.iter().cloned().filter(|p| p.y >= y_mid).collect();
        upper = resample_line(&upper, target_points);
        let mut lower: Vec<Point<f64>> =
            source.iter().cloned().filter(|p| p.y <= y_mid).collect();
        lower = resample_line(&lower, target_points);
        (upper, lower)
    }
    ///
    /// Отзеркаливание точек по Y и 
    /// - `points_to_mirror` - точки для отзеркаливания
    pub fn mirror_points(
        points_to_mirror: Vec<OPoint<f64, Const<3>>>,
    ) -> Vec<OPoint<f64, Const<3>>> {
        let mut result = Vec::new();
        for point in points_to_mirror {
            result.push(
                OPoint::<f64, Const<3>>::new(point.x, -point.y, point.z)
            );
        }
        result
    }   
    ///
    /// Переупорядочивание `points` в порядке `reference`.
    /// Подбирает для каждой точки из `reference` ближайшую (y,z) точку из `points` (без повторов).
    /// - `reference` - базисные точки
    /// - `points` - точки для упорядочивания
    pub fn order_points_like(
        reference: &[OPoint<f64, Const<3>>],
        points: &[OPoint<f64, Const<3>>],
    ) -> Vec<OPoint<f64, Const<3>>> {
        if reference.is_empty() || points.is_empty() {
            return Vec::new();
        }
        let mut used = vec![false; points.len()];
        let mut ordered = Vec::with_capacity(reference.len());
        for r in reference.iter() {
            let mut best_j: Option<usize> = None;
            let mut best_d2 = f64::INFINITY;
            for (j, p) in points.iter().enumerate() {
                if used[j] {
                    continue;
                }
                let dy = p.y - r.y;
                let dz = p.z - r.z;
                let d2 = dy * dy + dz * dz;
                if d2 < best_d2 {
                    best_d2 = d2;
                    best_j = Some(j);
                }
            }
            if let Some(j) = best_j {
                used[j] = true;
                ordered.push(points[j]);
            }
        }
        for (j, p) in points.iter().enumerate() {
            if !used[j] {
                ordered.push(*p);
            }
        }
        ordered
    }
    ///
    /// Отзеркаливание точек по Y и 
    /// добавление к векторам исходных вершин и индексов
    /// - `vertices` - вершины фигуры
    /// - `indices` - массив индексов треугольников фигуры
    /// - `mirror_mask` - для каких вершин нужно строить зеркальное отображение
    pub fn mirror_vert_ind(
        vertices: &mut Vec<OPoint<f64, Const<3>>>,
        indices: &mut Vec<[u32; 3]>,
        mirror_mask: &[bool],
    ) {
        const EPS: f64 = 0.0;
        let original_len = vertices.len();
        let mut remap = vec![0u32; original_len];
        for i in 0..original_len {
            let p = vertices[i];
            if !mirror_mask.get(i).copied().unwrap_or(true) || p.y.abs() == EPS {
                remap[i] = i as u32;
            } else {
                let mirrored = OPoint::<f64, Const<3>>::new(p.x, -p.y, p.z);
                let new_index = vertices.len() as u32;
                vertices.push(mirrored);
                remap[i] = new_index;
            }
        }
        let original_indices = indices.clone();
        for [a, b, c] in original_indices {
            let ma = remap[a as usize];
            let mb = remap[b as usize];
            let mc = remap[c as usize];
            if ma == a && mb == b && mc == c {
                continue;
            }
            indices.push([ma, mc, mb]);
        }
    }
    ///
    /// Соединение точек из
    /// двух блоков координат
    /// - `vertices` - набор вершин 3D фигуры
    /// - `indices` - набор индексов вершин 3D фигуры
    /// - `points_x1` - первый блок точек для соединения
    /// - `points_x2` - второй блок точек для соединения
    pub fn connect_points(
        vertices: &mut Vec<OPoint<f64, Const<3>>>,
        indices: &mut Vec<[u32; 3]>,
        points_x1: &[OPoint<f64, Const<3>>],
        points_x2: &[OPoint<f64, Const<3>>],
        reverse: bool,
        p_1: bool,
    ) {
        let n = points_x1.len();
        let mut base = vertices.len() as u32;
        if p_1 {
            for point in points_x1 {
                vertices.push(*point);
            }
        } else {
            base -= n as u32;
        }
        for point in points_x2 {
            vertices.push(*point);
        }
        let x1 = base;
        let x2 = base + n as u32;
        for i in 0..n {
            let next = (i + 1) % n;
            let a = x1 + i as u32;
            let b = x1 + next as u32;
            let c = x2 + next as u32;
            let d = x2 + i as u32;
            if reverse {
                if !is_degenerate(vertices, [a, c, b]) {
                    indices.push([a, c, b]);
                }
                if !is_degenerate(vertices, [a, d, c]) {
                    indices.push([a, d, c]);
                }
            } else {
                if !is_degenerate(vertices, [a, b, c]) {
                    indices.push([a, b, c]);
                }
                if !is_degenerate(vertices, [a, c, d]) {
                    indices.push([a, c, d]);
                }
            }
        }
    }
    ///
    /// Вычисление максимального значения
    /// координаты Y
    /// - `points` - точки для расчёта
    fn frame_max_y(points: &[OPoint<f64, Const<3>>]) -> f64 {
        let mut res = 0.0;
        for p in points {
            if p.z > res {
                res = p.z;
            }
        }
        res
    }
    ///
    /// Форматированние рубежного шпангоута
    /// для состыковки с предудыщем шпангоутом
    /// до изменения Y координаты
    /// - `raw_next_p` - сырые точки последнего шпангоута
    /// - `raw_prev_p` - сырые точки предыдущего шпангоута
    /// - `next_p` - интерполяционные точки последнего шпангоута
    /// - `prev_p` - интерполяционные точки предыдущего шпангоута
    /// 
    pub fn make_frame(
        raw_next_p: &[OPoint<f64, Const<3>>],
        raw_prev_p: &[OPoint<f64, Const<3>>],
        next_p: &[OPoint<f64, Const<3>>],
        prev_p: &[OPoint<f64, Const<3>>],
    ) -> (Vec<OPoint<f64, Const<3>>>, Vec<OPoint<f64, Const<3>>>) {
        let mut points_substract: Vec<OPoint<f64, Const<3>>> = Vec::new();
        let mut result = Vec::new();
        if Self::frame_max_y(raw_next_p) > Self::frame_max_y(raw_prev_p) {
            let mut base_point = None;
            for i in 0..raw_next_p.len() {
                if raw_next_p[i] != raw_prev_p[i] {
                    base_point = Some(raw_next_p[i - 1]);
                    break;
                }
            }
            let base = base_point.unwrap();
            for i in 0..prev_p.len() {
                if base == prev_p[i] {
                    for j in i..prev_p.len() {
                        points_substract.push(prev_p[j]);
                    }
                    break;
                }
                result.push(prev_p[i]);
            }
            let mut start = None;
            for i in 0..next_p.len() {
                if base == next_p[i] {
                    start = Some(i);
                    break;
                }
            }
            let start = start.unwrap();
            for j in (start + 1..next_p.len()).rev() {
                points_substract.push(next_p[j]);
            }
            result.extend_from_slice(&next_p[start..next_p.len()]);
        } else {
            let mut base_point = None;
            for i in 0..raw_prev_p.len() {
                if raw_next_p[i] != raw_prev_p[i] {
                    base_point = Some(raw_prev_p[i - 1]);
                    break;
                }
            }
            let base = base_point.unwrap();
            for i in 0..prev_p.len() {
                if base == prev_p[i] {
                    for j in i..prev_p.len() {
                        points_substract.push(prev_p[j]);
                    }
                    break;
                }
                result.push(prev_p[i]);
            }
            let mut start = None;
            for i in 0..next_p.len() {
                if base == next_p[i] {
                    start = Some(i);
                    break;
                }
            }
            let start = start.unwrap();
            for j in (start + 1..next_p.len()).rev() {
                points_substract.push(next_p[j]);
            }
            result.extend_from_slice(&next_p[start..next_p.len()]);
        }
        (points_substract, result)
    }
}