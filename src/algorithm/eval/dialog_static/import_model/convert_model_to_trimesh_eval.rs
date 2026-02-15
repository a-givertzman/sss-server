use std::io::Write;
use std::path::PathBuf;

use boolmesh::prelude::Manifold;
use nalgebra::{
    Const, 
    OPoint, Vector3
};
use boolmesh::{self, compute_boolean};
use parry3d_f64::math::Point;
use parry3d_f64::shape::TriMesh;
use sal_core::{
    dbg::Dbg, 
    error::Error
};
use crate::algorithm::context::context_access::ContextRead;
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
/// Write data to .stl file
pub fn write_stl(path: &PathBuf, mesh: &TriMesh) -> Result<(), Error> {
    let error = Error::new("Shape", "write_stl");
    let (result, empty_normals): (Vec<_>, Vec<_>) = mesh
        .triangles()
        .map(|t| (t.normal(), t))
        .partition(|(n, _)| n.is_some());
    if !empty_normals.is_empty() {
        return Err(error.err(format!("calculate normal error, path:{:?}", path)));
    }
    let triangles: Vec<_> = result
        .into_iter()
        .map(|(n, t)| {
            let n = n.unwrap();
            let normal = stl_io::Vector([n[0] as f32, n[1] as f32, n[2] as f32]);
            let vertices = [
                stl_io::Vector([t.a[0] as f32, t.a[1] as f32, t.a[2] as f32]),
                stl_io::Vector([t.b[0] as f32, t.b[1] as f32, t.b[2] as f32]),
                stl_io::Vector([t.c[0] as f32, t.c[1] as f32, t.c[2] as f32]),
            ];
            stl_io::Triangle { normal, vertices }
        })
        .collect();
    let mut binary_stl = Vec::<u8>::new();
    stl_io::write_stl(&mut binary_stl, triangles.iter())
        .map_err(|err| error.pass_with("stl_io::write_stl", err.to_string()))?;
    let mut buffer = std::fs::File::create(&path).map_err(|err| {
        error.pass_with(format!("File::create, path:{:?}", path), err.to_string())
    })?;
    buffer.write_all(&binary_stl).map_err(|err| {
        error.pass_with(
            format!("buffer.write_all, path:{:?}", path),
            err.to_string(),
        )
    })
}
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
                indices.push([a, c, b]);
                indices.push([a, d, c]);
            } else {
                indices.push([a, b, c]);
                indices.push([a, c, d]);
            }
        }
    }
    ///
    /// 
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
    /// 
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
    /// 
    fn calculate_polygon_area(&self, points: &[Point<f64>]) -> f64 {
        if points.len() < 3 {
            return 0.0;
        }
        let mut area = 0.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            area += points[i].x * points[j].z - points[j].x * points[i].z;
        }
        area.abs() / 2.0
    }
    ///
    /// Отзеркаливание точек по Y и 
    /// добавление к векторам исходных вершин и индексов
    fn mirror_points(
        &self,
        vertices: &mut Vec<OPoint<f64, Const<3>>>,
        indices: &mut Vec<[u32; 3]>,
    ) {
        const EPS: f64 = 0.0;
        let original_len = vertices.len();
        let mut remap = vec![0u32; original_len];
        for i in 0..original_len {
            let p = vertices[i];
            if p.y.abs() == EPS {
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
    /// Создание [Manifold]
    /// - `vertices` - вершины фигуры
    /// - `indices` - массив индексов треугольников фигуры
    fn create_manifold(&self, vertices: &[OPoint<f64, Const<3>>], indices: &[[u32; 3]]) -> Result<Manifold,Error> {
        let mut all_coords = Vec::new();
        for point in vertices {
            all_coords.push(point.x);
            all_coords.push(point.y);
            all_coords.push(point.z);
        }
        let mut all_indx = Vec::new();
        for triangle_indx in indices {
            for indx in triangle_indx {
                all_indx.push(*indx as usize);
            }
        }
        match Manifold::new(&all_coords, &all_indx) {
            Ok(manifold) => {
                return Ok(manifold)
            },
            Err(e) => return Err(e.into()),
        }
    }
    ///
    /// Преобразование [Manifold] в [TriMesh]
    /// - `manifold` - [Manifold] для преобразования
    fn manifold_to_trimesh(&self, manifold: Manifold) -> Result<TriMesh, Error> {
        let vertices: Vec<OPoint<f64, Const<3>>> =
            manifold.ps.iter()
                .map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z))
                .collect();
        let indices: Vec<[u32; 3]> =
            manifold.get_indices().iter()
                .map(|t| [t[0] as u32, t[1] as u32, t[2] as u32])
                .collect();
        TriMesh::new(vertices, indices)
            .map_err(|e| e.to_string().into())
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
    /// Создание и индексирование вершин
    /// - `tanks_3d` - набор блоков координат отсеков
    fn convert_surface_outer(&self, tanks_3d: Import3DModelCtx) -> TriMesh {
        let target_points = 600;
        let mut result: Vec<TriMesh> = Vec::new();
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut start_wall = None;
        let mut p1_flag = false;
        for i in 0..tanks_3d.surface_outer_body.coordinates.len() - 1 {
            let curr_frame = &tanks_3d.surface_outer_body.coordinates[i];
            let next_frame = &tanks_3d.surface_outer_body.coordinates[i + 1];
            let points_x1 = self.resample_line(
                &self.convert_to_points_vec(curr_frame), 
                target_points
            );
            let points_x2 = self.resample_line(
                &self.convert_to_points_vec(next_frame), 
                target_points
            );
            if start_wall.is_none() {
                start_wall = Some(points_x1.clone());
            }
            if curr_frame[0].0 == next_frame[0].0 {
                let start = start_wall.as_ref().unwrap();
                self.build_wall(
                false,
                    0,
                    &mut all_indices,
                    &mut all_vertices,
                    &start,
                    start.len()
                );
                self.build_wall(
                true,
                    (all_vertices.len() - points_x1.len() - 1) as u32,
                    &mut all_indices,
                    &mut all_vertices,
                    &points_x1,
                    points_x1.len()
                );
                match TriMesh::new(all_vertices.clone(), all_indices.clone()) { // последняя часть
                    Ok(tri) => {
                        result.push(tri)

                    },
                    Err(e) => log::error!("Failed to write nasal mesh {}", e),
                }  
                all_indices = Vec::new();
                all_vertices = Vec::new();
                start_wall = None;
                p1_flag = true;
                continue;
            } 
            else if i == tanks_3d.surface_outer_body.coordinates.len() - 2 { // конец модели
                let start = start_wall.as_ref().unwrap();
                self.connect_points(
                    &mut all_vertices, 
                    &mut all_indices, 
                    &points_x1, 
                    &points_x2, 
                    true,
                    false
                );
                self.build_wall(
                false,
                    0,
                    &mut all_indices,
                    &mut all_vertices,
                    &start,
                    start.len()
                );
                self.build_wall(
                false,
                    (all_vertices.len() - points_x2.len() + 1) as u32,
                    &mut all_indices,
                    &mut all_vertices,
                    &points_x2,
                    points_x2.len()
                );  
                match TriMesh::new(all_vertices.clone(), all_indices.clone()) { // последняя часть
                    Ok(tri) => {
                        result.push(tri)

                    },
                    Err(e) => log::error!("Failed to write nasal mesh {}", e),
                }  
            }
            if i == 0 || p1_flag {
                self.connect_points(
                    &mut all_vertices, 
                    &mut all_indices, 
                    &points_x1, 
                    &points_x2, 
                    true,
                    true,
                );
                p1_flag = false;
            } else {
                self.connect_points(
                    &mut all_vertices, 
                    &mut all_indices, 
                    &points_x1, 
                    &points_x2, 
                    true,
                    false,
                );                
            }
        }
        let mut last_manifold = None;
        for tri in result {
            match self.create_manifold(&tri.vertices(), &tri.indices()) {
                Ok(ad) => {
                    if last_manifold.is_none() {
                        last_manifold = Some(ad);
                    } else {
                        match compute_boolean(&ad, &last_manifold.clone().unwrap(),  boolmesh::prelude::OpType::Intersect) {
                            Ok(add) => {
                                last_manifold = Some(add.clone());
                                match self.manifold_to_trimesh(add) {
                                    Ok(tr) => {
                                        let path = PathBuf::from(format!("src/tests/unit/algorithm/dialog_static/output_files/1.stl"));
                                        if let Err(e) = write_stl(&path, &tr) {
                                            log::error!("Failed to write nasal mesh {}", e);
                                        }
                                    },
                                    Err(e) => log::error!("Failed to write nasal mesh {}", e),
                                }
                            },
                            Err(e) => log::error!("Failed to write nasal mesh {}", e),
                        }
                    }
                },
                Err(e) => log::error!("Failed to write nasal mesh {}", e)
            }           
        }
        match self.manifold_to_trimesh(last_manifold.unwrap()) {
            Ok(tri) => return tri,
            Err(e) => panic!("Failed to write nasal mesh {}", e),
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
                let model_3d = ContextRead::<Import3DModelCtx>::read(&ctx).clone();
                let surface_outer_body = self
                    .convert_surface_outer(model_3d);
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