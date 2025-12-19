use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use std::thread::current;
use nalgebra::Const;
use nalgebra::OPoint;
use nalgebra::Vector3;
use parry3d_f64::math::Point;
use parry3d_f64::shape::TriMesh;
use parry3d_f64::shape::Triangle;
use sal_core::{dbg::Dbg, error::Error};
use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::eval::entities::surface_outer_body::SurfaceOuterBody;
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
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertModelToTrimeshEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
    ///
    /// Преобразование баттокса (корма/нос)
    fn convert_buttocks(&self, buttocks: Vec<(f64, f64)>, target_points: usize) -> Option<TriMesh> {
        if buttocks.len() < 3 {
            log::warn!("{} | Not enough points for buttocks: {}", self.dbg, buttocks.len());
            return None;
        }
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for (z, x) in &buttocks {
            vertices.push(Point::new(*x, 0.0, *z));
        }
        vertices = self.resample_line(&vertices, target_points);
        let (wall_vertices, wall_indices) = self.close_frame_end(&vertices, None);
        let base_index = vertices.len() as u32;
        for vertex in wall_vertices {
            vertices.push(vertex);
        }
        for triangle in wall_indices {
            let idx1 = (triangle[0] + base_index) as usize;
            let idx2 = (triangle[1] + base_index) as usize;
            let idx3 = (triangle[2] + base_index) as usize;
            if idx1 < vertices.len() && idx2 < vertices.len() && idx3 < vertices.len() {
                let p1 = vertices[idx1];
                let p2 = vertices[idx2];
                let p3 = vertices[idx3];
                match Triangle::new(p1, p2, p3).normal() {
                    Some(normal) => {
                        if normal.norm() > 1e-10 {
                            indices.push([triangle[0] + base_index, triangle[1] + base_index, triangle[2] + base_index]);
                        } else {
                            log::warn!("Zero-length normal detected in wall triangle");
                        }
                    },
                    None => {
                        log::warn!("Failed to compute normal for wall triangle");
                    },
                }
            } else {
                log::error!("Index out of bounds in wall triangle: {} {} {}", idx1, idx2, idx3);
            }
        }
        match TriMesh::new(vertices, indices) {
            Ok(final_mesh) => Some(final_mesh),
            Err(err) => {
                log::error!("Failed to create mirrored TriMesh: {}", err);
                None
            }
        }
    }
    ///
    /// Интерполяция фрейма до одинакового количества точек
    fn resample_line(&self, points: &[Point<f64>], n: usize) -> Vec<Point<f64>> {
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
    /// Закрытие одного сэмпла веером
    fn close_frame_end(&self, points: &[Point<f64>], mut centroid: Option<OPoint<f64, Const<3>>>) -> (Vec<Point<f64>>, Vec<[u32; 3]>) {
        let mut vertices = points.to_vec();
        let mut indices = Vec::new();
        if points.len() < 3 {
            return (vertices, indices);
        }
        if centroid.is_none() { centroid = Some(self.calculate_centroid(&points)) }
        let center_index = vertices.len() as u32;
        vertices.push(centroid.unwrap());
        for i in 0..points.len() {
            let next_i = (i + 1) % points.len();
            indices.push([center_index, i as u32, next_i as u32]);
        }
        (vertices, indices)
    }
    ///
    /// Вычисляется центроида сэмпла
    fn calculate_centroid(&self, points: &[Point<f64>]) -> Point<f64> {
        let sum = points.iter().fold(Point::new(0.0, 0.0, 0.0), |acc, p| {
            Point::new(acc.x + p.x, acc.y + p.y, acc.z + p.z)
        });
        let count = points.len() as f64;
        Point::new(sum.x / count, sum.y / count, sum.z / count)

    }
    ///
    /// Сохранение точки
    fn save_point_to_txt(point: &Point<f64>, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "{:.6} {:.6} {:.6}", point.x, point.y, point.z)?;
        Ok(())
    }
    ///
    /// Сохранение точек
    fn save_points_to_txt(points: &[Point<f64>], path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for p in points {
            writeln!(writer, "{:.6} {:.6} {:.6}", p.x, p.y, p.z)?;
        }
        Ok(())
    }
    ///
    /// Соединение шпангоутов
    pub fn get_vertices_indeces(&self, frames: Vec<(f64, Vec<Point<f64>>)>, nasal_block: Option<TriMesh>, main_decks_indices: Vec<usize>) -> (Vec<Point<f64>>, Vec<[u32; 3]>, Vec<TriMesh>) {
        let mut result: Vec<TriMesh> = Vec::new();
        let mut vertices: Vec<Point<f64>> = Vec::new();
        let mut indices: Vec<[u32; 3]> = Vec::new();
        let mut start_trimesh: Option<usize> = None;
        for i in 0..frames.len() - 1 {
            if start_trimesh.is_none() { start_trimesh = Some(i); }
            if &frames[i].0 == &frames[i + 1].0 {
                // let mut previous = frames[start_trimesh.unwrap()].1.clone();
                // let mut current = frames[start_trimesh.unwrap()].1.clone();
                // vertices = Vec::new();
                // indices = Vec::new();
                // start_trimesh = None;

            } else {
                let mut previous = Vec::new();
                let mut current = Vec::new();
                if !main_decks_indices.contains(&i) &&  !main_decks_indices.contains(&(i + 1)) {
                    previous = frames[i].1.clone();
                    current = frames[i + 1].1.clone();
                    let mut j_prev = 0;
                    let mut j_curr = 0;
                    while j_prev < previous.len()-1 && j_curr < current.len()-1 {
                        let p1: OPoint<f64, Const<3>> = previous[j_prev];
                        let p2 = previous[j_prev+1];
                        let c1 = current[j_curr];
                        let c2 = current[j_curr+1];
                        let global_base = vertices.len() as u32;
                        let mut local_vertices = Vec::new();
                        let mut local_indices = Vec::new();
                        local_vertices.push(p1);
                        local_vertices.push(p2);
                        local_vertices.push(c1);
                        local_vertices.push(c2);
                        local_indices.push([0, 1, 2]);
                        local_indices.push([1, 3, 2]);
                        // переносим их в глобальные
                        for v in local_vertices.clone() {
                            vertices.push(v);
                        }
                        for [a,b,c] in local_indices.clone() {
                            indices.push([
                                a + global_base,
                                b + global_base,
                                c + global_base
                            ]);
                        }
                        j_prev += 1;
                        j_curr += 1;
                    }
                    let previous_reverse: Vec<Point<f64>> = previous.iter().map(| point | {
                        OPoint::<f64, Const<3>>::new(point.x, -point.y, point.z)
                    }).collect();
                    let current_reverse: Vec<Point<f64>> = current.iter().map(| point | {
                        OPoint::<f64, Const<3>>::new(point.x, -point.y, point.z)
                    }).collect();
                    j_prev = 0;
                    j_curr = 0;
                    while j_prev < previous_reverse.len()-1 && j_curr < current_reverse.len()-1 {
                        let p1: OPoint<f64, Const<3>> = previous_reverse[j_prev];
                        let p2 = previous_reverse[j_prev+1];
                        let c1 = current_reverse[j_curr];
                        let c2 = current_reverse[j_curr+1];
                        let global_base = vertices.len() as u32;
                        let mut local_vertices = Vec::new();
                        let mut local_indices = Vec::new();
                        local_vertices.push(p1);
                        local_vertices.push(p2);
                        local_vertices.push(c1);
                        local_vertices.push(c2);
                        local_indices.push([0, 1, 2]);
                        local_indices.push([1, 3, 2]);
                        // переносим их в глобальные
                        for v in local_vertices.clone() {
                            vertices.push(v);
                        }
                        for [a,b,c] in local_indices.clone() {
                            indices.push([
                                a + global_base,
                                b + global_base,
                                c + global_base
                            ]);
                        }
                        j_prev += 1;
                        j_curr += 1;
                    }
                } else if !main_decks_indices.contains(&i) && main_decks_indices.contains(&(i + 1)) { 
                    previous = frames[i].1.clone();
                    current = frames[i + 1].1.clone();
                    // --- Делим current пополам по оси Y ---
                    // Находим среднее значение Y
                    let mut y_sum = 0.0;
                    for p in &current {
                        y_sum += p.y;
                    }
                    let y_mid = y_sum / current.len() as f64;
                    // Разделяем на две части
                    let mut upper: Vec<Point<f64>> = Vec::new();
                    for p in current {
                        if p.y <= y_mid {
                            continue;
                        } else {
                            upper.push(p);
                        }
                    }
                    upper = self.resample_line(&upper, 600);
                    let mut lower: Vec<Point<f64>> = upper.iter().map(| point | {
                        OPoint::<f64, Const<3>>::new(point.x, -point.y, point.z)
                    }).collect();
                    // Сортируем upper по координате X так же, как previous
                    upper.sort_by(|a, b| {
                        let a_index = previous.iter().position(|p| (p.x - a.x).abs() < 1e-8).unwrap_or(0);
                        let b_index = previous.iter().position(|p| (p.x - b.x).abs() < 1e-8).unwrap_or(0);
                        a_index.cmp(&b_index)
                    });
                    let mut j_prev = 0;
                    let mut j_curr = 0;
                    // первая половина
                    while j_prev < previous.len() - 1 && j_curr < upper.len() - 1 {
                        let p1: OPoint<f64, Const<3>> = previous[j_prev];
                        let p2 = previous[j_prev + 1];
                        let c1 = upper[j_curr];
                        let c2 = upper[j_curr + 1];
                        let global_base = vertices.len() as u32;
                        let local_vertices = vec![p1, p2, c1, c2];
                        let local_indices = vec![[0, 1, 2], [1, 3, 2]];
                        // переносим в глобальные
                        for v in local_vertices.clone() {
                            vertices.push(v);
                        }
                        for [a, b, c] in local_indices {
                            indices.push([a + global_base, b + global_base, c + global_base]);
                        }

                        j_prev += 1;
                        j_curr += 1;
                    }
                    // вторая половина
                    let mut previous_reverse: Vec<Point<f64>> = previous.iter().map(| point | {
                        OPoint::<f64, Const<3>>::new(point.x, -point.y, point.z)
                    }).collect();
                    previous_reverse.sort_by(|a , b| a.x.partial_cmp(&b.x).unwrap());
                    // Сортируем upper по координате X так же, как previous_reverse
                    lower.sort_by(|a, b| {
                        let a_index = previous_reverse.iter().position(|p| (p.x - a.x).abs() < 1e-8).unwrap_or(0);
                        let b_index = previous_reverse.iter().position(|p| (p.x - b.x).abs() < 1e-8).unwrap_or(0);
                        a_index.cmp(&b_index)
                    });
                    let mut j_prev = 0;
                    let mut j_curr = 0;
                    // первая половина
                    while j_prev < previous_reverse.len() - 1 && j_curr < lower.len() - 1 {
                        let p1: OPoint<f64, Const<3>> = previous_reverse[j_prev];
                        let p2 = previous_reverse[j_prev + 1];
                        let c1 = lower[j_curr];
                        let c2 = lower[j_curr + 1];
                        let global_base = vertices.len() as u32;
                        let local_vertices = vec![p1, p2, c1, c2];
                        let local_indices = vec![[0, 1, 2], [1, 3, 2]];
                        // переносим в глобальные
                        for v in local_vertices.clone() {
                            vertices.push(v);
                        }
                        for [a, b, c] in local_indices {
                            indices.push([a + global_base, b + global_base, c + global_base]);
                        }
                        j_prev += 1;
                        j_curr += 1;
                    }
                } else if main_decks_indices.contains(&i) && !main_decks_indices.contains(&(i + 1)) {
                    previous = frames[i].1.clone();
                    current = frames[i + 1].1.clone();
                    // --- Делим current пополам по оси Y ---
                    // Находим среднее значение Y
                    let mut y_sum = 0.0;
                    for p in &previous {
                        y_sum += p.y;
                    }
                    let y_mid = y_sum / previous.len() as f64;
                    // Разделяем на две части
                    let mut upper: Vec<Point<f64>> = Vec::new();
                    for p in previous {
                        if p.y <= y_mid {
                            continue;
                        } else {
                            upper.push(p);
                        }
                    }
                    upper = self.resample_line(&upper, 600);
                    let mut lower: Vec<Point<f64>> = upper.iter().map(| point | {
                        OPoint::<f64, Const<3>>::new(point.x, -point.y, point.z)
                    }).collect();
                    // Сортируем upper по координате X так же, как previous
                    upper.sort_by(|a, b| {
                        let a_index = current.iter().position(|p| (p.x - a.x).abs() < 1e-8).unwrap_or(0);
                        let b_index = current.iter().position(|p| (p.x - b.x).abs() < 1e-8).unwrap_or(0);
                        a_index.cmp(&b_index)
                    });
                    let mut j_prev = 0;
                    let mut j_curr = 0;
                    // первая половина
                    while j_prev < current.len() - 1 && j_curr < upper.len() - 1 {
                        let p1: OPoint<f64, Const<3>> = current[j_prev];
                        let p2 = current[j_prev + 1];
                        let c1 = upper[j_curr];
                        let c2 = upper[j_curr + 1];
                        let global_base = vertices.len() as u32;
                        let local_vertices = vec![p1, p2, c1, c2];
                        let local_indices = vec![[0, 1, 2], [1, 3, 2]];
                        // переносим в глобальные
                        for v in local_vertices.clone() {
                            vertices.push(v);
                        }
                        for [a, b, c] in local_indices {
                            indices.push([a + global_base, b + global_base, c + global_base]);
                        }

                        j_prev += 1;
                        j_curr += 1;
                    }
                    // вторая половина
                    let mut current_reverse: Vec<Point<f64>> = current.iter().map(| point | {
                        OPoint::<f64, Const<3>>::new(point.x, -point.y, point.z)
                    }).collect();
                    current_reverse.sort_by(|a , b| a.x.partial_cmp(&b.x).unwrap());
                    // Сортируем upper по координате X так же, как current_reverse
                    lower.sort_by(|a, b| {
                        let a_index = current_reverse.iter().position(|p| (p.x - a.x).abs() < 1e-8).unwrap_or(0);
                        let b_index: usize = current_reverse.iter().position(|p| (p.x - b.x).abs() < 1e-8).unwrap_or(0);
                        a_index.cmp(&b_index)
                    });
                    let mut j_prev = 0;
                    let mut j_curr = 0;
                    // первая половина
                    while j_prev < current_reverse.len() - 1 && j_curr < lower.len() - 1 {
                        let p1: OPoint<f64, Const<3>> = current_reverse[j_prev];
                        let p2 = current_reverse[j_prev + 1];
                        let c1 = lower[j_curr];
                        let c2 = lower[j_curr + 1];
                        let global_base = vertices.len() as u32;
                        let local_vertices = vec![p1, p2, c1, c2];
                        let local_indices = vec![[0, 1, 2], [1, 3, 2]];
                        // переносим в глобальные
                        for v in local_vertices.clone() {
                            vertices.push(v);
                        }
                        for [a, b, c] in local_indices {
                            indices.push([a + global_base, b + global_base, c + global_base]);
                        }
                        j_prev += 1;
                        j_curr += 1;
                    }
                } else if main_decks_indices.contains(&i) && main_decks_indices.contains(&(i + 1)) {
                    previous = frames[i].1.clone();
                    current = frames[i + 1].1.clone();
                    let mut j_prev = 0;
                    let mut j_curr = 0;
                    // первая половина
                    while j_prev < previous.len() - 1 && j_curr < current.len() - 1 {
                        let p1: OPoint<f64, Const<3>> = previous[j_prev];
                        let p2 = previous[j_prev + 1];
                        let c1 = current[j_curr];
                        let c2 = current[j_curr + 1];
                        let global_base = vertices.len() as u32;
                        let local_vertices = vec![p1, p2, c1, c2];
                        let local_indices = vec![[0, 1, 2], [1, 3, 2]];
                        // переносим в глобальные
                        for v in local_vertices.clone() {
                            vertices.push(v);
                        }
                        for [a, b, c] in local_indices {
                            indices.push([a + global_base, b + global_base, c + global_base]);
                        }
                        j_prev += 1;
                        j_curr += 1;
                    }
                }
            }
        }
        let mut start_frame = frames[start_trimesh.unwrap()].1.clone();
        let mut end_frame = frames.last().unwrap().1.clone();
        if !main_decks_indices.contains(&start_trimesh.unwrap()) {
            let start_frame_reverse: Vec<Point<f64>> = start_frame.iter().map(| point | {
                OPoint::<f64, Const<3>>::new(point.x, -point.y, point.z)
            }).collect();
            start_frame.extend(start_frame_reverse);
        }
        if !main_decks_indices.contains(&(frames.len() - 1)) {
            let end_frame_reverse: Vec<Point<f64>> = end_frame.iter().map(| point | {
                OPoint::<f64, Const<3>>::new(point.x, -point.y, point.z)
            }).collect();
            end_frame.extend(end_frame_reverse);
        }
        let start_wall = self.close_frame_end(&start_frame, Some(self.calculate_centroid(&start_frame)));
        let end_wall = self.close_frame_end(&end_frame, Some(self.calculate_centroid(&end_frame)));
        // match TriMesh::new(start_wall.0.clone(), start_wall.1.clone()) {
        //     Ok(_trimesh) => {
        //         vertices.extend(start_wall.0);
        //         indices.extend(start_wall.1);
        //     }
        //     Err(err) => {
        //         log::error!("Failed to create TriMesh: {}", err);
        //     }
        // }
        // match TriMesh::new(end_wall.0.clone(), end_wall.1.clone()) {
        //     Ok(_trimesh) => {
        //         vertices.extend(end_wall.0);
        //         indices.extend(end_wall.1);
        //     }
        //     Err(err) => {
        //         log::error!("Failed to create TriMesh: {}", err);
        //     }
        // }
        match TriMesh::new(vertices.clone(), indices.clone()) {
            Ok(_trimesh) => {
                result.push(_trimesh);
            }
            Err(err) => {
                log::error!("Failed to create TriMesh: {}", err);
            }
        }
        (vertices, indices, result)
    }
    ///
    /// Преобразование поверхности основной модели
    fn convert_surface_outer(&self, surface: SurfaceOuterBody, target_points: usize, nasal_block: Option<TriMesh>) -> Option<Vec<TriMesh>> {
        let mut result: Vec<TriMesh> = Vec::new();
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
        for (_, verts) in frames.iter_mut() {
            *verts = self.resample_line(verts, target_points);
        }
        let (vertices, indices, res) = self.get_vertices_indeces(frames, nasal_block, surface.main_deck);
        result = res;
        match TriMesh::new(vertices.clone(), indices.clone()) {
            Ok(trimesh) => {
                result.push(trimesh);
                Some(result)
            }
            Err(err) => {
                log::error!("Failed to create TriMesh: {}", err);
                Some(result)
            }
        }
    }
    ///
    /// Преобразование поверхности надстроек
    fn convert_surface_superstructure(&self, surface: Vec<Vec<(f64,f64,f64)>>, target_points: usize) -> Option<TriMesh> {
        if surface.len() < 2 {
            return  None;
        }
        let mut frames: Vec<(f64, Vec<Point<f64>>)> = Vec::new();
        for vertices in &surface {
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
        for (_, verts) in frames.iter_mut() {
            *verts = self.resample_line(verts, target_points);
        }
        let mut vertices: Vec<Point<f64>> = Vec::new();
        let mut indices: Vec<[u32; 3]> = Vec::new();
        if let Some(first_frame) = frames.first() {
            let (wall_vertices, wall_indices) = self.close_frame_end(&first_frame.1, None);
            let base_index = vertices.len() as u32;
            for vertex in wall_vertices {
                vertices.push(vertex);
            }
            for triangle in wall_indices {
                let idx1 = (triangle[0] + base_index) as usize;
                let idx2 = (triangle[1] + base_index) as usize;
                let idx3 = (triangle[2] + base_index) as usize;
                if idx1 < vertices.len() && idx2 < vertices.len() && idx3 < vertices.len() {
                    let p1 = vertices[idx1];
                    let p2 = vertices[idx2];
                    let p3 = vertices[idx3];
                    match Triangle::new(p1, p2, p3).normal() {
                        Some(normal) => {
                            if normal.norm() > 1e-10 {
                                indices.push([triangle[0] + base_index, triangle[1] + base_index, triangle[2] + base_index]);
                            } else {
                                log::warn!("Zero-length normal detected in wall triangle");
                            }
                        },
                        None => {
                            log::warn!("Failed to compute normal for wall triangle");
                        },
                    }
                } else {
                    log::error!("Index out of bounds in wall triangle: {} {} {}", idx1, idx2, idx3);
                }
            }
        }
        for i in 0..frames.len() - 1 {
            let previous = &frames[i].1;
            let current = &frames[i + 1].1;
            let mut j_prev = 0;
            let mut j_curr = 0;
            while j_prev < previous.len()-1 && j_curr < current.len()-1 {
                let p1: OPoint<f64, Const<3>> = previous[j_prev];
                let p2 = previous[j_prev+1];
                let c1 = current[j_curr];
                let c2 = current[j_curr+1];
                let base = vertices.len() as u32;
                vertices.push(p1);
                vertices.push(p2);
                vertices.push(c1);
                vertices.push(c2);
                if Triangle::new(p1, p2, c1).normal().is_some() {
                    indices.push([base, base+1, base+2]);
                }
                if Triangle::new(p2, c2, c1).normal().is_some() {
                    indices.push([base+1, base+3, base+2]);
                }
                j_prev += 1;
                j_curr += 1;
            }
        }
        if let Some(last_frame) = frames.last() {
            let (wall_vertices, wall_indices) = self.close_frame_end(&last_frame.1, None);
            let base_index = vertices.len() as u32;
            for vertex in wall_vertices {
                vertices.push(vertex);
            }
            for triangle in wall_indices {
                let idx1 = (triangle[0] + base_index) as usize;
                let idx2 = (triangle[1] + base_index) as usize;
                let idx3 = (triangle[2] + base_index) as usize;
                if idx1 < vertices.len() && idx2 < vertices.len() && idx3 < vertices.len() {
                    let p1 = vertices[idx1];
                    let p2 = vertices[idx2];
                    let p3 = vertices[idx3];
                    match Triangle::new(p1, p2, p3).normal() {
                        Some(normal) => {
                            if normal.norm() > 1e-10 {
                                indices.push([triangle[0] + base_index, triangle[1] + base_index, triangle[2] + base_index]);
                            } else {
                                log::warn!("Zero-length normal detected in wall triangle");
                            }
                        },
                        None => {
                            log::warn!("Failed to compute normal for wall triangle");
                        },
                    }
                } else {
                    log::error!("Index out of bounds in wall triangle: {} {} {}", idx1, idx2, idx3);
                }
            }
        }
        match TriMesh::new(vertices.clone(), indices.clone()) {
            Ok(_trimesh) => {
                let original_vertex_count = vertices.len() as u32;
                let mirrored_vertices: Vec<Point<f64>> = vertices
                    .iter()
                    .map(|p| Point::new(p.x, -p.y, p.z))
                    .collect();
                let mirrored_indices: Vec<[u32; 3]> = indices
                    .iter()
                    .map(|[a, b, c]| [a + original_vertex_count, c + original_vertex_count, b + original_vertex_count])
                    .collect();
                let mut all_vertices = vertices.clone();
                all_vertices.extend(mirrored_vertices);
                let mut all_indices = indices.clone();
                all_indices.extend(mirrored_indices);
                match TriMesh::new(all_vertices, all_indices) {
                    Ok(final_mesh) => Some(final_mesh),
                    Err(err) => {
                        log::error!("Failed to create mirrored TriMesh: {}", err);
                        None
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to create TriMesh: {}", err);
                None
            }
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
                let target_points = 600; // чем больше тем лучше модель: Кол-во точек на сэмпл для интерполяции
                let stern_block = self
                    .convert_buttocks(model_3d.stern_block.coordinates.clone(), target_points);
                let nasal_block = self
                    .convert_buttocks(model_3d.nasal_block.coordinates.clone(), target_points);
                let surface_outer_body = self
                    .convert_surface_outer(model_3d.surface_outer_body.clone(), target_points, nasal_block.clone());
                let surface_superstructure = self
                    .convert_surface_superstructure(model_3d.surface_superstructure.coordinates.clone(), target_points);
                ctx.write(ConvertModelToTrimeshCtx {
                    stern_block: stern_block,
                    nasal_block: nasal_block,
                    surface_outer_body,
                    surface_superstructure: surface_superstructure,
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
