use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use nalgebra::{
    Const, 
    OPoint, Vector3
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
use crate::algorithm::eval::import_model::convert_model_to_trimesh_ctx::ConvertModelToTrimeshCtx;
use crate::algorithm::eval::import_model::import_3d_model_ctx::Import3DModelCtx;
use crate::main;
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
    /// Новый экземпляр [ConvertModelToTrimeshEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertModelToTrimeshEval");
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
    // ///
    // /// 
    // fn resample_line_with_gap(&self, points: &Vec<OPoint<f64, Const<3>>>, gap: f64) -> Vec<OPoint<f64, Const<3>>> {
    //     let mut result = Vec::new();
    //     if points.is_empty() {
    //         return result;
    //     }
    //     result.push(points[0]);
    //     for i in 0..points.len() - 1 {
    //         let curr = points[i];
    //         let next = points[i + 1];
    //         let dir = next - curr;
    //         let dist = (dir.x.powi(2) + dir.y.powi(2) + dir.z.powi(2)).sqrt();
    //         if dist > gap {
    //             let num_steps = (dist / gap).floor() as i32;
    //             for k in 1..=num_steps {
    //                 let t = (k as f64) * gap / dist;
    //                 if (t - 1.0).abs() < 1e-12 {
    //                     break;
    //                 }
    //                 let point = OPoint::<f64, Const<3>>::new(
    //                     curr.x + t * dir.x,
    //                     curr.y + t * dir.y,
    //                     curr.z + t * dir.z,
    //                 );
    //                 result.push(point);
    //             }
    //         }
    //         result.push(next);
    //     }
    //     result
    // }
    // ///
    // /// 
    // fn euclid_dist(&self, a: OPoint<f64, Const<3>>, b: OPoint<f64, Const<3>>) -> f64 {
    //     return ((a.y - b.y) + (a.z - b.z)).powf(0.5);
    // }
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
        // боковые торцы
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
    /// Закрытие сэмпла без добавления центральной вершины
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
        _points: &Vec<OPoint<f64, Const<3>>>, 
        n: usize,
    ) {
        if n < 3 {
            return;
        }
        let base = x2;
        for i in 1..(n - 1) {
            let a = base;
            let b = base + i as u32;
            let c = base + (i + 1) as u32;
            if reverse {
                if !self.is_degenerate(vertices, [a, c, b]) {
                    indices.push([a, c, b]);
                }
            } else {
                if !self.is_degenerate(vertices, [a, b, c]) {
                    indices.push([a, b, c]);
                }
            }
        }
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
    /// Переупорядочивание `points` в порядке `reference`.
    /// Подбирает для каждой точки из `reference` ближайшую (y,z) точку из `points` (без повторов).
    fn order_points_like(
        &self,
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

        // если были лишние точки (на всякий случай) — добавим в конец
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
    fn mirror_vert_ind(
        &self,
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
        (upper, lower)
    }
    ///
    /// Соединение шпангоутов ГП (главной палубы)
    /// - `main_deck` - набор шпангоутов ГП
    fn connect_main_deck(
        &self,
        main_deck: Vec<Vec<OPoint<f64, Const<3>>>>,
        target_points: usize
    ) -> TriMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut prev_points: Option<Vec<OPoint<f64, Const<3>>>> = None;
        for i in 0..main_deck.len() {
            let frame = &main_deck[i];
            let opoints: Vec<OPoint<f64, Const<3>>> = self.resample_line(&frame, target_points);
            if i == 0 {
                vertices.extend_from_slice(&opoints);
            } else {
                let prev_x = main_deck[i-1][0].x;
                let curr_x = frame[0].x;
                if prev_x == curr_x {
                    if let Some(prev) = &prev_points {
                        self.connect_points(
                            &mut vertices,
                            &mut indices,
                            prev,
                            &opoints,
                            true,
                            false,
                        );
                    }
                    prev_points = Some(opoints);
                    continue;
                } else {
                    if let Some(prev) = &prev_points {
                        self.connect_points(
                            &mut vertices,
                            &mut indices,
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
        match TriMesh::new(vertices, indices) {
            Ok(trimesh) => trimesh,
            Err(e) => panic!("Error to connect main deck: {:?}", e),
        }
    }
    ///
    /// Сохранение точек
    fn save_points_to_txt(points: &[Point<f64>], path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter  ::new(file);
        for p in points {
            writeln!(writer, "{:.6} {:.6} {:.6}", p.x, p.y, p.z)?;
        }
        Ok(())
    }
    ///
    /// Создание и индексирование вершин
    /// - `tanks_3d` - набор блоков координат отсеков
    fn convert_surface_outer(&self, tanks_3d: Import3DModelCtx, target_points: usize) -> TriMesh {
        let mut all_vertices: Vec<OPoint<f64, Const<3>>> = Vec::new();
        let mut all_indices: Vec<[u32; 3]> = Vec::new();
        let mut mirror_mask: Vec<bool> = Vec::new();
        let mut prev_points: Option<Vec<OPoint<f64, Const<3>>>> = None;
        let mut main_deck = Vec::new();
        let mut flag_last_main_deck = false; // мы сейчас внутри диапазона main_deck
        let mut last_lower_gapped: Option<Vec<OPoint<f64, Const<3>>>> = None;
        let mut gap = 0.0; // размер смещения для одинаковых шпангоутов
        for i in 0..tanks_3d.surface_outer_body.coordinates.len() {
            let frame: &Vec<(f64, f64, f64)> = &tanks_3d.surface_outer_body.coordinates[i];
            let points_vec = self.convert_to_points_vec(frame);
            let is_main_deck = tanks_3d.surface_outer_body.main_deck.contains(&i);
            let points: Vec<OPoint<f64, Const<3>>> = if is_main_deck {
                let (upper, lower) = self.split_and_resample(&points_vec, target_points);
                let lower_gapped: Vec<OPoint<f64, Const<3>>> = lower.into_iter().map(|p| OPoint::<f64, Const<3>>::new(p.x + gap, p.y, p.z)).collect();
                if !flag_last_main_deck {
                    if let Some(prev) = prev_points.as_ref() {
                        let prev_mirror = self.mirror_points(prev.clone());
                        let prev_mirror_ordered = self.order_points_like(&lower_gapped, &prev_mirror);
                        main_deck.push(prev_mirror_ordered);
                    }
                    flag_last_main_deck = true;
                }
                last_lower_gapped = Some(lower_gapped.clone());
                main_deck.push(lower_gapped);   
                upper.into_iter().map(|p| OPoint::<f64, Const<3>>::new(p.x + gap, p.y, p.z)).collect()
            } else {
                self.resample_line(&points_vec, target_points)
            };
            let opoints: Vec<_> = points.iter().map(|p| OPoint::<f64, Const<3>>::new(p.x + gap, p.y, p.z)).collect();
            // первый фрейм после main_deck (переход main_deck: true -> false)
            if !is_main_deck && flag_last_main_deck {
                if let Some(reference) = last_lower_gapped.as_ref() {
                    let after_mirror = self.mirror_points(opoints.clone());
                    let after_mirror_ordered = self.order_points_like(reference, &after_mirror);
                    main_deck.push(after_mirror_ordered);
                } else {
                    main_deck.push(self.mirror_points(opoints.clone()));
                }
                flag_last_main_deck = false;
            }
            if i == 0 {
                all_vertices.extend_from_slice(&opoints);
                mirror_mask.extend(std::iter::repeat(!is_main_deck).take(target_points));
                if let Some(first) = all_vertices.clone().get(0..target_points) {
                    self.build_wall(false, 0, &mut all_indices, &mut all_vertices, &first.to_vec(), target_points); // корма
                }
            } else {
                let prev_x = tanks_3d.surface_outer_body.coordinates[i-1][0].0;
                let curr_x = frame[0].0;
                if prev_x == curr_x {
                    gap += 1e-9;
                    let gapped_points: Vec<OPoint<f64, Const<3>>> = opoints.iter().map(|p| OPoint::<f64, Const<3>>::new(p.x + gap, p.y, p.z)).collect();
                    if let Some(prev) = &prev_points {
                        self.connect_points(
                            &mut all_vertices,
                            &mut all_indices,
                            prev,
                            &gapped_points,
                            true,
                            false,
                        );
                    }
                    mirror_mask.extend(std::iter::repeat(!is_main_deck).take(target_points));
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
                    mirror_mask.extend(std::iter::repeat(!is_main_deck).take(target_points));
                }
            }
            prev_points = Some(opoints);
        }
        let last_start = all_vertices.len() - target_points;
        if let Some(last) = all_vertices.clone().get(last_start..) {
            self.build_wall(true, last_start as u32, &mut all_indices, &mut all_vertices, &last.to_vec(), target_points); // нос
        }
        self.mirror_vert_ind(&mut all_vertices, &mut all_indices, &mirror_mask);
        match TriMesh::new(all_vertices, all_indices) {
            Ok(mut ship_model) => {
                let _ = ship_model.set_flags(TriMeshFlags::MERGE_DUPLICATE_VERTICES);
                let _ = ship_model.set_flags(TriMeshFlags::DELETE_DUPLICATE_TRIANGLES);
                let _ = ship_model.set_flags(TriMeshFlags::DELETE_DEGENERATE_TRIANGLES);
                let _ = ship_model.set_flags(TriMeshFlags::DELETE_BAD_TOPOLOGY_TRIANGLES);
                let _ = ship_model.set_flags(TriMeshFlags::FIX_INTERNAL_EDGES);
                let _ = ship_model.set_flags(TriMeshFlags::ORIENTED);
                if !main_deck.is_empty() {
                    let mut main_deck = self.connect_main_deck(main_deck, target_points);
                    let _ = main_deck.set_flags(TriMeshFlags::MERGE_DUPLICATE_VERTICES);
                    let _ = main_deck.set_flags(TriMeshFlags::DELETE_DUPLICATE_TRIANGLES);
                    let _ = main_deck.set_flags(TriMeshFlags::DELETE_DEGENERATE_TRIANGLES);
                    let _ = main_deck.set_flags(TriMeshFlags::DELETE_BAD_TOPOLOGY_TRIANGLES);
                    let _ = main_deck.set_flags(TriMeshFlags::FIX_INTERNAL_EDGES);
                    let _ = main_deck.set_flags(TriMeshFlags::ORIENTED);
                    ship_model.append(&main_deck);
                    let _ = ship_model.set_flags(TriMeshFlags::MERGE_DUPLICATE_VERTICES);
                    let _ = ship_model.set_flags(TriMeshFlags::DELETE_DUPLICATE_TRIANGLES);
                    let _ = ship_model.set_flags(TriMeshFlags::DELETE_DEGENERATE_TRIANGLES);
                    let _ = ship_model.set_flags(TriMeshFlags::DELETE_BAD_TOPOLOGY_TRIANGLES);
                    let _ = ship_model.set_flags(TriMeshFlags::FIX_INTERNAL_EDGES);
                    let _ = ship_model.set_flags(TriMeshFlags::ORIENTED);
                }
                ship_model
            },
            Err(e) => panic!("Error to create ship model: {}",e ),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for ConvertModelToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let target_points = 600; // кол-во точек для интерполяции
                let model_3d = ContextRead::<Import3DModelCtx>::read(&ctx).clone();
                let surface_outer_body = self
                    .convert_surface_outer(model_3d, target_points);
                ctx.write(
                    ConvertModelToTrimeshCtx {
                        stern_block: None,
                        nasal_block: None,
                        surface_outer_body: Some(surface_outer_body),
                        surface_superstructure: None,
                    }
                )
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