use nalgebra::{
    Const, 
    OPoint
};
use parry3d_f64::math::Point;
use parry3d_f64::shape::{
    TriMesh, 
    TriMeshFlags
};
use sal_core::{
    dbg::Dbg, 
    error::Error
};
use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::eval::import_model::convert_surface_superstructure_to_trimesh::convert_surface_superstructure_to_trimesh_ctx::ConvertSurfaceSuperStructureToTrimeshCtx;
use crate::algorithm::eval::import_model::import_model_initial_points::import_model_initial_points_ctx::ImportModelInitialPointsCtx;
use crate::{
    algorithm::eval::{Zg},
    kernel::{
        eval::Eval,
        types::eval_result::EvalResult
    },
    prelude::ContextWrite,
};
///
/// Преобразование координат 3D модели в тип данных TriMesh
pub struct ConvertSurfaceSuperStructureToTrimeshEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ConvertSurfaceSuperStructureToTrimeshEval {
    ///
    /// Новый экземпляр [ConvertSurfaceSuperStructureToTrimeshEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertSurfaceSuperStructureToTrimeshEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
    ///
    /// Преобразование координат в набор точек [Point]
    /// - `vec_x_y_z` - набор (X,Y,Z) координат
    fn convert_to_points_vec(
        &self,
        vec_x_y_z: &Vec<(f64, f64, f64)>, 
    ) -> Vec<Point<f64>>{
        let mut frame: Vec<Point<f64>> = Vec::new();
        for (x, y, z) in vec_x_y_z {
            let point = Point::new(*x, *z, *y);
            frame.push(point);
        }
        return frame;
    }
    ///
    /// Проверка на вырожденность треугольника
    /// - `vertices` - набор вершин треугольника
    /// - `tri` - индексы вершин треугольника
    fn is_degenerate(
        &self,
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
    ///
    /// Интерполяция фрейма до одинакового количества точек
    /// - `points` - набор точек для интерполяции
    /// - `n` - кол-во точек интерполяции
    fn resample_line(
        &self,
        points: &[Point<f64>],
        n: usize,
    ) -> Vec<Point<f64>> {
        if points.len() < 2 || n <= points.len() {
            return points.to_vec();
        }
        let m = points.len();
        let segs = m - 1;
        let mut lengths = Vec::with_capacity(segs);
        let mut total_len = 0.0;
        for i in 0..segs {
            let a = points[i];
            let b = points[i + 1];
            let d = ((b.x - a.x).powi(2)
                + (b.y - a.y).powi(2)
                + (b.z - a.z).powi(2))
                .sqrt();
    
            lengths.push(d);
            total_len += d;
        }
        let extra = n - m;
        let mut extras_per_seg = vec![0usize; segs];
        for i in 0..segs {
            extras_per_seg[i] =
                ((lengths[i] / total_len) * extra as f64).round() as usize;
        }
        let mut sum: usize = extras_per_seg.iter().sum();
        while sum > extra {
            for e in &mut extras_per_seg {
                if *e > 0 {
                    *e -= 1;
                    sum -= 1;
                    if sum == extra { break; }
                }
            }
        }
        while sum < extra {
            for e in &mut extras_per_seg {
                *e += 1;
                sum += 1;
                if sum == extra { break; }
            }
        }
        let mut result = Vec::with_capacity(n);
        for i in 0..segs {
            let a = points[i];
            let b = points[i + 1];
            result.push(a);
            let k = extras_per_seg[i];
            for j in 1..=k {
                let t = j as f64 / (k + 1) as f64;
                let p = Point::new(
                    a.x + (b.x - a.x) * t,
                    a.y + (b.y - a.y) * t,
                    a.z + (b.z - a.z) * t,
                );
                result.push(p);
            }
        }
        result.push(*points.last().unwrap());
        result
    }
    ///
    /// Соединение точек из
    /// двух блоков координат
    /// - `vertices` - набор вершин 3D фигуры
    /// - `indices` - набор индексов вершин 3D фигуры
    /// - `points_x1` - первый блок точек для соединения
    /// - `points_x2` - второй блок точек для соединения
    fn connect_points(
        &self,
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
                if !self.is_degenerate(vertices, [a, c, b]) {
                    indices.push([a, c, b]);
                }
                if !self.is_degenerate(vertices, [a, d, c]) {
                    indices.push([a, d, c]);
                }
            } else {
                if !self.is_degenerate(vertices, [a, b, c]) {
                    indices.push([a, b, c]);
                }
                if !self.is_degenerate(vertices, [a, c, d]) {
                    indices.push([a, c, d]);
                }
            }
        }
    }
    ///
    /// Закрытие сэмпла веером
    /// - `reverse` - ориентация нормалей
    /// - `x2` - индекс начала вершин закрываемого сэмпла
    /// - `vertices` - вершины 3D модели
    /// - `indices` - массив индексов треугольников фигуры
    /// - `points` - вершины закрываемого сэмпла
    /// - `n` - кол-во точек закрываемого сэмпла
    fn build_wall(
        &self, 
        reverse: bool, 
        x2: u32, 
        indices: &mut Vec<[u32; 3]>, 
        vertices: &mut Vec<OPoint<f64, Const<3>>>,
        points: &Vec<OPoint<f64, Const<3>>>, 
        n: usize,
    ) {
        let centroid = self.calculate_centroid(points);
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
        &self, 
        points: &[Point<f64>]
    ) -> Point<f64> {
        let sum = points.iter().fold(Point::new(0.0, 0.0, 0.0), |acc, p| {
            Point::new(acc.x + p.x, acc.y + p.y, acc.z + p.z)
        });
        let count = points.len() as f64;
        Point::new(sum.x / count, sum.y / count, sum.z / count)
    }
    ///
    /// Отзеркаливание точек по Y и 
    /// - `points_to_mirror` - точки для отзеркаливания
    fn mirror_points(
        &self,
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
    /// Вычисление максимального значения
    /// координаты Y
    /// - `points` - точки для расчёта
    fn frame_max_y(&self, points: &[OPoint<f64, Const<3>>]) -> f64 {
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
    fn make_frame(
        &self,
        raw_next_p: &[OPoint<f64, Const<3>>],
        raw_prev_p: &[OPoint<f64, Const<3>>],
        next_p: &[OPoint<f64, Const<3>>],
        prev_p: &[OPoint<f64, Const<3>>],
    ) -> (Vec<OPoint<f64, Const<3>>>, Vec<OPoint<f64, Const<3>>>) {
        let mut points_substract: Vec<OPoint<f64, Const<3>>> = Vec::new();
        let mut result = Vec::new();
        if self.frame_max_y(raw_next_p) > self.frame_max_y(raw_prev_p) {
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
    ///
    /// Создание и индексирование вершин
    /// - `tanks_3d` - набор блоков координат отсеков
    /// - `target_points` - кол-во точек на шпангоут для интерполяции
    fn convert_surface_superstructure(&self, tanks_3d: ImportModelInitialPointsCtx, mut target_points: usize) -> Option<TriMesh> {
        let mut all_vertices: Vec<OPoint<f64, Const<3>>> = Vec::new();
        let mut all_indices: Vec<[u32; 3]> = Vec::new();
        let mut walls_diff_y = Vec::new();
        let mut prev_points: Option<Vec<OPoint<f64, Const<3>>>> = None;
        let mut first = Vec::new();
        for i in 0..tanks_3d.surface_superstructure.len() {
            let frame: &Vec<(f64, f64, f64)> = &tanks_3d.surface_superstructure[i];
            let points_vec = self.convert_to_points_vec(frame);
            let points: Vec<OPoint<f64, Const<3>>> = self.resample_line(&points_vec, target_points);
            if i == 0 {
                first = points.clone();
            }
            let opoints: Vec<_> = points.iter().map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z)).collect();
            if all_vertices.len() == 0 {
                all_vertices.extend_from_slice(&opoints);
            } else {
                let prev_x = tanks_3d.surface_superstructure[i-1][0].0;
                let curr_x = frame[0].0;
                if prev_x == curr_x {
                    let mut gapped_points: Vec<OPoint<f64, Const<3>>> = opoints.iter().map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z)).collect();
                    if let Some(prev) = &prev_points {
                        let ress = self.make_frame(&points_vec, &self.convert_to_points_vec(&tanks_3d.surface_superstructure[i - 1]), &gapped_points, prev);
                        gapped_points = ress.1;
                        let mut wall_vert = Vec::new();
                        wall_vert.extend_from_slice(&ress.0.to_vec());
                        let mut wall_ind = Vec::new();
                        self.build_wall(true, 0 as u32, &mut wall_ind, &mut wall_vert, &ress.0.to_vec(),  ress.0.len() - 1); // нос
                        walls_diff_y.push(TriMesh::new(wall_vert, wall_ind).expect("Error to build wall"));
                    }
                    all_vertices.extend_from_slice(&gapped_points);
                    target_points = gapped_points.len();
                    prev_points = Some(gapped_points);
                    continue;
                } else {
                    if let Some(prev) = &prev_points {
                        self.connect_points(
                            &mut all_vertices,
                            &mut all_indices,
                            prev,
                            &opoints,
                            true,
                            false,
                        );
                    }
                }
            }
            prev_points = Some(opoints);
        }
        self.build_wall(false, 0, &mut all_indices, &mut all_vertices, &first.to_vec(), first.len()); // корма
        let last_start = all_vertices.len() - target_points - 1;
        if let Some(last) = all_vertices.clone().get(last_start..) {
            self.build_wall(true, last_start as u32, &mut all_indices, &mut all_vertices, &last.to_vec(),  last.len() - 1); // нос
        }
        match TriMesh::new(all_vertices, all_indices) {
            Ok(mut ship_model) => {
                for wall in walls_diff_y {
                    ship_model.append(&wall);
                }
                let _ = ship_model.set_flags(TriMeshFlags::MERGE_DUPLICATE_VERTICES);
                let _ = ship_model.set_flags(TriMeshFlags::DELETE_DUPLICATE_TRIANGLES);
                let _ = ship_model.set_flags(TriMeshFlags::DELETE_DEGENERATE_TRIANGLES);
                let _ = ship_model.set_flags(TriMeshFlags::DELETE_BAD_TOPOLOGY_TRIANGLES);
                let _ = ship_model.set_flags(TriMeshFlags::FIX_INTERNAL_EDGES);
                let _ = ship_model.set_flags(TriMeshFlags::ORIENTED);
                Some(ship_model)
            },
            Err(e) => panic!("Error to create ship model: {}",e ),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for ConvertSurfaceSuperStructureToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let target_points = 200 ; // кол-во точек для интерполяции
                let model_3d = ContextRead::<ImportModelInitialPointsCtx>::read(&ctx).clone();
                let surface_outer_body = self
                    .convert_surface_superstructure(model_3d, target_points);
                ctx.write(
                    ConvertSurfaceSuperStructureToTrimeshCtx {
                        result: surface_outer_body,
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ConvertSurfaceSuperStructureToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertSurfaceSuperStructureToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}