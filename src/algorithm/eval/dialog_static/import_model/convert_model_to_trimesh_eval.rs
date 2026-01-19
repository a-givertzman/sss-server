use nalgebra::Const;
use nalgebra::OPoint;
use nalgebra::Vector3;
use parry3d_f64::math::Point;
use parry3d_f64::shape::TriMesh;
use sal_core::{
    dbg::Dbg, 
    error::Error
};
use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::eval::entities::surface_outer_body::SurfaceOuterBody;
use crate::algorithm::eval::entities::surface_superstructure::SurfaceSuperstructure;
use crate::algorithm::eval::import_model::convert_model_to_trimesh_ctx::ConvertModelToTrimeshCtx;
use crate::algorithm::eval::import_model::import_3d_model_ctx::Import3DModelCtx;
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
pub struct ConvertModelToTrimeshEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ConvertModelToTrimeshEval {
    ///
    /// Новый экземпляр класса [ConvertModelToTrimeshEval]
    pub fn new(
        parent: impl Into<String>, 
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static
    ) -> Self {
        let dbg = Dbg::new(parent, "ConvertModelToTrimeshEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
    ///
    /// Преобразование баттокса (корма/нос)
    /// - 'buttocks' - набор точек баттокса
    /// - `target_points` - кол-во точек интерполяции
    fn convert_buttocks(
        &self, 
        buttocks: Vec<(f64, f64)>, 
        target_points: usize
    ) -> Option<TriMesh> {
        if buttocks.len() < 3 {
            log::warn!("{} | Not enough points for buttocks: {}", self.dbg, buttocks.len());
            return None;
        }
        let mut vertices = Vec::new();
        for (z, x) in &buttocks {
            vertices.push(Point::new(*x, 0.0, *z));
        }
        vertices = self.resample_line(&vertices, target_points);
        if let Some(buttocks) =  self.close_frame_end(vertices) {
            match TriMesh::new(buttocks.0, buttocks.1) {
                Ok(final_mesh) => return Some(final_mesh),
                Err(err) => {
                    log::error!("Failed to create mirrored TriMesh: {}", err);
                    return None;
                }
            }
        }
        None
    }
    ///
    /// Интерполяция фрейма до одинакового количества точек
    /// - `points` - набор точек для интерполяции
    /// - `n` - кол-во точек интерполяции
    fn resample_line(
        &self, 
        points: &[Point<f64>], 
        n: usize
    ) -> Vec<Point<f64>> {
        if points.len() < 2 || n < 2 {
            return points.to_vec();
        }
        let mut lengths = vec![0.0];
        for i in 0..points.len() - 1 {
            let prev = points[i];
            let curr = points[i + 1];
            let d = ((curr.x - prev.x).powi(2)
                + (curr.y - prev.y).powi(2)
                + (curr.z - prev.z).powi(2))
                .sqrt();
            lengths.push(lengths.last().unwrap() + d);
        }
        let total_len = *lengths.last().unwrap();
        if total_len == 0.0 {
            return points.to_vec();
        }
        let step = total_len / (n - 1) as f64;
        let mut result = Vec::with_capacity(n);
        let mut idx = 0;
        for i in 0..n {
            let target = step * i as f64;
            while idx + 1 < lengths.len() && lengths[idx + 1] < target {
                idx += 1;
            }
            if idx + 1 == lengths.len() {
                result.push(points.last().unwrap().clone());
                continue;
            }
            let l1 = lengths[idx];
            let l2 = lengths[idx + 1];
            let t = (target - l1) / (l2 - l1);
            let mut use_corner_point = false;
            let mut corner_point = None;
            if idx + 2 < points.len() {
                let a = points[idx];
                let b = points[idx + 1];
                let c = points[idx + 2];
                let ba = Vector3::new(a.x - b.x, a.y - b.y, a.z - b.z);
                let bc = Vector3::new(c.x - b.x, c.y - b.y, c.z - b.z);
                
                let dot = ba.dot(&bc);
                let eps = 1e-6;
                if dot.abs() < eps {
                    let corner_pos = lengths[idx + 1];
                    if (target - corner_pos).abs() < step * 0.1 {
                        use_corner_point = true;
                        corner_point = Some(b);
                    }
                }
            }
            if use_corner_point {
                result.push(corner_point.unwrap().clone());
            } else {
                let p1 = points[idx];
                let p2 = points[idx + 1];
                let interpolated_point = Point::new(
                    p1.x + (p2.x - p1.x) * t,
                    p1.y + (p2.y - p1.y) * t,
                    p1.z + (p2.z - p1.z) * t,
                );
                result.push(interpolated_point);
            }
        }
        if result.len() > n {
            result.truncate(n);
        } else if result.len() < n {
            while result.len() < n {
                result.push(points.last().unwrap().clone());
            }
        }
        result
    }
    ///
    /// Создание стены из сэмпла
    /// - `frame` - шпангоут, который надо закрыть стенкой
    /// - `result` - набор готовых TriMesh
    fn build_wall(
        &self, 
        frame: Vec<Point<f64>>, 
        result: &mut Vec<TriMesh>
    )  {
        match self.close_frame_end(frame) {
            Some((vertices, indices)) => {
                match TriMesh::new(vertices, indices) {
                    Ok(trimesh) => {
                        result.push(trimesh);
                    },
                    Err(err) => {
                        log::error!("Failed to create TriMesh: {}", err);
                    },
                }
            },
            None => {
                log::debug!("Error to create a wall!");
            },
        }
    }
    ///
    /// Заливка сэмпла веером
    /// - `points` - точки для заливки
    fn close_frame_end(
        &self, 
        points: Vec<Point<f64>>, 
    ) -> Option<(Vec<Point<f64>>, Vec<[u32; 3]>)> {
        let mut vertices = points.to_vec();
        let mut indices = Vec::new();
        if points.len() < 3 {
            return Some((vertices, indices));
        }
        let centroid = self.calculate_centroid(&points);
        let center_index = vertices.len() as u32;
        vertices.push(centroid);
        for i in 0..points.len() - 1 {
            let idx1 = i as u32;
            let idx2 = (i + 1) as u32;
            if self.is_valid_triangle(&points[i], &points[i + 1], &centroid) {
                if points[i].y <= 0.0 {
                    indices.push([center_index, idx1, idx2]);
                } else {
                    indices.push([center_index, idx2, idx1]);
                }
            } 
        }
        if indices.is_empty() {
            log::warn!("No valid triangles created in close_frame_end");
            return None;
        }
        Some((vertices, indices))
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
    /// Отзеркаливание по координате Y
    /// - `line` - то что надо отзеркалить
    fn mirror_y(
        &self, 
        line: &[Point<f64>]
    ) -> Vec<Point<f64>> {
        line.iter()
            .map(|p| OPoint::<f64, Const<3>>::new(p.x, -p.y, p.z))
            .collect()
    }
    ///
    /// Проверка треугольника на валидность
    /// - `p1` - точка треугольника
    /// - `p2` - точка треугольника
    /// - `p3` - точка треугольника
    fn is_valid_triangle(
        &self, 
        p1: &Point<f64>, 
        p2: &Point<f64>, 
        p3: &Point<f64>
    ) -> bool {
        let eps = 1e-10;

        // Проверка на уникальность вершин
        if (p1 - p2).norm_squared() < eps ||
           (p2 - p3).norm_squared() < eps ||
           (p3 - p1).norm_squared() < eps {
            return false;
        }
        // Проверка на коллинеарность
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let cross = v1.cross(&v2);
        cross.norm_squared() > eps
    }
    ///
    /// Квадрат из двух треугольников
    /// 3D фигура
    /// - `vertices` - набор вершин
    /// - `indices` - набор индексов вершин
    /// - `p1` - точка для соединения
    /// - `p2` - точка для соединения
    /// - `c1` - точка для соединения
    /// - `c2` - точка для соединения
    fn push_quad(
        &self,
        vertices: &mut Vec<Point<f64>>,
        indices: &mut Vec<[u32; 3]>,
        p1: Point<f64>,
        p2: Point<f64>,
        c1: Point<f64>,
        c2: Point<f64>,
        reverse: bool,
    ) {
        let base = vertices.len() as u32;
        vertices.extend([p1, p2, c1, c2]);
        if reverse {
            indices.extend([
                // [base, base + 2, base + 1],
                // [base + 1, base + 2, base + 3],
                [base, base + 2, base + 3],
                [base, base + 3, base + 1],
            ]);
        } else {
            indices.extend([
                // [base, base + 1, base + 2],
                // [base + 1, base + 3, base + 2],
                [base, base + 3, base + 2],
                [base, base + 1, base + 3],
            ]);
        }
    }
    ///
    /// Соединение двух сэмплов
    /// - `vertices` - набор вершин
    /// - `indices` - набор индексов вершин
    /// - `a` - шпангоут 1
    /// - `b` - шпангоут 2
    /// - `reverse` - маркирова для реверсивного соединения
    fn stitch_lines(
        &self, 
        vertices: &mut Vec<Point<f64>>,
        indices: &mut Vec<[u32; 3]>,
        a: &[Point<f64>],
        b: &[Point<f64>],
        reverse: bool,
    ) {
        let n = a.len().min(b.len());
        for i in 0..n.saturating_sub(1) {
            self.push_quad(vertices, indices, a[i], a[i + 1], b[i], b[i + 1], reverse);
        }
    }
    ///
    /// Разбиение сэмпла пополам
    /// с последующей интерполяцией
    /// по всей длине
    /// - `source` - сэмпл для разбиения
    /// - `target_points` - кол-во точек на сэмпл для интерполяции
    fn split_and_resample(
        &self,
        source: &[Point<f64>],
        target_points: usize,
    ) -> (Vec<Point<f64>>, Vec<Point<f64>>) {
        let mut source_sort_by_y = source.to_vec();
        source_sort_by_y.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
        let y_mid = source_sort_by_y[source_sort_by_y.len() / 2].y;
        let mut upper: Vec<Point<f64>> =
            source.iter().cloned().filter(|p| p.y >= y_mid).collect();
        upper = self.resample_line(&upper, target_points);
        let mut lower: Vec<Point<f64>> =
            source.iter().cloned().filter(|p| p.y <= y_mid).collect();
        lower = self.resample_line(&lower, target_points);
        let lower = self.mirror_y(&upper);
        (upper, lower)
    }
    ///
    /// Создание вершин и их индексирование
    /// для 3D модели
    /// - `frames` - исходные шпангоуты
    /// - `nasal_block` - модель носового баттокса
    /// - `target_points` - кол-во точек на сэмпл для интерполяции
    pub fn get_vertices_indeces(
        &self,
        frames: Vec<(f64, Vec<Point<f64>>)>,
        nasal_block: Option<TriMesh>,
        main_decks_indices: Vec<usize>,
        target_points: usize,
    ) -> TriMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut result = Vec::new();
        let mut start_trimesh = None;
        for i in 0..frames.len().saturating_sub(1) {
            start_trimesh.get_or_insert(i);
            if frames[i].0 == frames[i + 1].0 {
                // начальная стена
                if !main_decks_indices.contains(&start_trimesh.unwrap()) {
                    let start_frame = frames[start_trimesh.unwrap()].1.clone();
                    let start_frame_mirr = self.mirror_y(&frames[start_trimesh.unwrap()].1.clone());
                    let mut all_points = Vec::new();
                    all_points.extend_from_slice(&start_frame);
                    all_points.extend_from_slice(&start_frame_mirr);
                    self.build_wall(all_points, &mut result);
                } else {
                    let start_frame = frames[start_trimesh.unwrap()].1.clone();
                    self.build_wall(start_frame, &mut result);
                }
                // конечная стена
                if !main_decks_indices.contains(&start_trimesh.unwrap()) {
                    let curr_end_frame = frames[i].1.clone();
                    let curr_end_frame_mirr = self.mirror_y(&frames[i].1.clone());
                    let mut all_points = Vec::new();
                    all_points.extend_from_slice(&curr_end_frame);
                    all_points.extend_from_slice(&curr_end_frame_mirr);
                    self.build_wall(all_points, &mut result);
                } else {
                    let curr_end_frame = frames[i].1.clone();
                    self.build_wall(curr_end_frame, &mut result);
                }
                // часть корабля
                match TriMesh::new(vertices, indices) {
                    Ok(ship_part) => {
                        result.push(ship_part);
                    },
                    Err(err) => {
                        log::error!("Failed to create TriMesh: {}", err);
                    }
                }
                vertices = Vec::new();
                indices = Vec::new();
                start_trimesh = None;
                continue;
            }
            let mut prev = frames[i].1.clone();
            let mut curr = frames[i + 1].1.clone();
            if prev.len() != curr.len() {
                prev = self.resample_line(&prev, 600);
                curr = self.resample_line(&curr, 600);
            } 
            let is_main_i = main_decks_indices.contains(&i);
            let is_main_j = main_decks_indices.contains(&(i + 1));
            match (is_main_i, is_main_j) {
                (false, false) => {
                    self.stitch_lines(&mut vertices, &mut indices, &prev, &curr, true);
                    let prev_reverse = self.mirror_y(&prev);
                    let curr_reverse = self.mirror_y(&curr);
                    self.stitch_lines(
                        &mut vertices,
                        &mut indices,
                        &prev_reverse,
                        &curr_reverse,
                        false
                    );
                }
                (false, true) => {
                    let (upper, lower) = self.split_and_resample(&curr, target_points);
                    self.stitch_lines(&mut vertices, &mut indices, &prev, &upper, true);
                    let prev_rev = self.mirror_y(&prev);
                    self.stitch_lines(&mut vertices, &mut indices, &prev_rev, &lower, false);
                }
                (true, false) => {
                    let (upper, lower) = self.split_and_resample(&prev, target_points);
                    self.stitch_lines(&mut vertices, &mut indices, &curr, &upper, false);
                    let curr_rev = self.mirror_y(&curr);
                    self.stitch_lines(&mut vertices, &mut indices, &curr_rev, &lower, true);
                }
                (true, true) => {
                    self.stitch_lines(&mut vertices, &mut indices, &prev, &curr, true);
                }
            }
        }
        // начальная стена
        match start_trimesh {
            Some(start_trimesh) => {
                if !main_decks_indices.contains(&start_trimesh) {
                    let start_frame = frames[start_trimesh].1.clone();
                    let start_frame_mirr = self.mirror_y(&frames[start_trimesh].1.clone());
                    let mut all_points = Vec::new();
                    all_points.extend_from_slice(&start_frame);
                    all_points.extend_from_slice(&start_frame_mirr);
                    self.build_wall(all_points, &mut result);
                } else {
                    let start_frame = frames[start_trimesh].1.clone();
                    self.build_wall(start_frame, &mut result);
                }
            },
            None => {},
        }
        // конечная стена
        if !main_decks_indices.contains(&start_trimesh.unwrap()) {
            let curr_end_frame = frames[frames.len() - 1].1.clone();
            let curr_end_frame_mirr = self.mirror_y(&frames[frames.len() - 1].1.clone());
            let mut all_points = Vec::new();
            all_points.extend_from_slice(&curr_end_frame);
            all_points.extend_from_slice(&curr_end_frame_mirr);
            self.build_wall(all_points, &mut result);
        } else {
            let curr_end_frame = frames[frames.len() - 1].1.clone();
            self.build_wall(curr_end_frame, &mut result);
        }
        // часть корабля
        match TriMesh::new(vertices, indices) {
            Ok(ship_part) => {
                result.push(ship_part);
            },
            Err(err) => {
                log::error!("Failed to create TriMesh: {}", err);
            }
        }
        // объединение всех частей модели в одну
        let mut full_mesh = result.first().clone().unwrap().to_owned();
        for mesh in 1..result.len() {
            if result[mesh].vertices().len() > 0 {
                full_mesh.append(&result[mesh]);
            }
        }
        full_mesh
    }
    ///
    /// Преобразование поверхности основной модели
    /// - `surface` - экземпляр [SurfaceOuterBody]
    /// - `target_points` - кол-во точек на сэмпл для интерполяции
    fn convert_surface_outer(
        &self, 
        surface: SurfaceOuterBody, 
        target_points: usize, 
        nasal_block: Option<TriMesh>
    ) -> Option<TriMesh> {
        let mut frames: Vec<(f64, Vec<Point<f64>>)> = Vec::new();
        for vertices in &surface.coordinates {
            if vertices.is_empty() {;
                continue;
            }
            let frame = vertices[0].0;
            let points: Vec<Point<f64>> = vertices
            .iter()
            .map(|&(_, z, y)| Point::new(frame, y, z))
            .collect();
            frames.push((frame, points));
        }
        frames.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let res = self.get_vertices_indeces(frames, nasal_block, surface.main_deck, target_points);
        return Some(res);
    }
    ///
    /// Преобразование поверхности надстроек
    /// - `surface` - экземпляр [SurfaceSuperstructure]
    /// - `target_points` - кол-во точек на сэмпл для интерполяции
    fn convert_surface_superstructure(
        &self, 
        surface: SurfaceSuperstructure, 
        target_points: usize
    ) -> Option<TriMesh> {
        let mut frames: Vec<(f64, Vec<Point<f64>>)> = Vec::new();
        for vertices in &surface.coordinates {
            if vertices.is_empty() {
                continue;
            }
            let frame = vertices[0].0;
            let points: Vec<Point<f64>> = vertices
            .iter()
            .map(|&(_, z, y)| Point::new(frame, y, z))
            .collect();
            frames.push((frame, points));
        }
        frames.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let res = self.get_vertices_indeces(frames, None, Vec::new(), target_points);
        Some(res)
    }
}
//
//
impl Eval<Zg, EvalResult> for ConvertModelToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let model_3d = ContextRead::<Import3DModelCtx>::read(&ctx).clone();
                let target_points = 600; // Кол-во точек на сэмпл для интерполяции (качество модели)
                let stern_block = self
                    .convert_buttocks(model_3d.stern_block.coordinates.clone(), target_points);
                let nasal_block = self

                    .convert_buttocks(model_3d.nasal_block.coordinates.clone(), target_points);
                let surface_outer_body = self
                    .convert_surface_outer(model_3d.surface_outer_body.clone(), target_points, nasal_block.clone());
                // let surface_superstructure = self
                    // .convert_surface_superstructure(model_3d.surface_superstructure.clone(), target_points);
                ctx.write(ConvertModelToTrimeshCtx {
                    stern_block: stern_block,
                    nasal_block: nasal_block,
                    surface_outer_body,
                    surface_superstructure: None,
                })
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ConvertModelToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertModelToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
