use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use nalgebra::Const;
use nalgebra::OPoint;
use nalgebra::Vector3;
use parry3d_f64::math::Point;
use parry3d_f64::shape::TriMesh;
use parry3d_f64::shape::Triangle;
use sal_core::{dbg::Dbg, error::Error};
use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::eval::import_tanks::convert_tanks_to_trimesh_ctx::ConvertTanksToTrimeshCtx;
use crate::algorithm::eval::import_tanks::import_3d_tanks_ctx::Import3DTanksCtx;
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
pub struct ConvertTanksToTrimeshEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ConvertTanksToTrimeshEval {
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertTanksToTrimeshEval");
        Self { dbg, ctx: Box::new(ctx) }
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
    /// Преобразование координат угловых точек переборок отсеков
    fn convert_compartments_corner_points(&self, surface: Vec<Vec<(f64,f64,f64)>>, target_points: usize) -> Option<Vec<TriMesh>> {
        let mut result: Vec<TriMesh> = Vec::new();
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
        let mut start_trimesh: Option<usize> = None;
        for i in 0..frames.len() - 1 {
            if start_trimesh.is_none() { start_trimesh = Some(i); }
            if &frames[i].0 == &frames[i + 1].0 {
                // НАЧАЛЬНАЯ СТЕНКА
                let (wall_vertices, wall_indices) = self.close_frame_end(&frames[start_trimesh.unwrap()].1, None);
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
                                    indices.push([triangle[2] + base_index, triangle[1] + base_index, triangle[0] + base_index]);
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
                start_trimesh = None;
                // КОНЕЧНАЯ СТЕНКА 
                let (wall_vertices, wall_indices) = self.close_frame_end(&frames[i].1, None);
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
                                    indices.push([triangle[2] + base_index, triangle[1] + base_index, triangle[0] + base_index]);
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
                            Ok(final_mesh) => {
                                result.push(final_mesh);
                            },
                            Err(err) => {
                                log::error!("Failed to create mirrored TriMesh: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to create TriMesh: {}", err);
                    }
                }
                vertices = Vec::new();
                indices = Vec::new();
            } else {
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
        }
        // начальная стенка последнего сэмпла
        let (wall_vertices, wall_indices) = self.close_frame_end(&frames[start_trimesh.unwrap()].1, None);
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
                            indices.push([triangle[2] + base_index, triangle[1] + base_index, triangle[0] + base_index]);
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
        // стенка
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
                    Ok(final_mesh) => {
                        result.push(final_mesh);
                        Some(result)
                    },
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
impl Eval<Zg, EvalResult> for ConvertTanksToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let tanks_3d = ContextRead::<Import3DTanksCtx>::read(&ctx).clone();
                let target_points = 600; // чем больше тем лучше модель: Кол-во точек на сэмпл для интерполяции
                let compartment_corner_points = self
                    .convert_compartments_corner_points(tanks_3d.compartment_corner_points.coordinates.clone(), target_points);
                ctx.write(ConvertTanksToTrimeshCtx {
                    compartment_corner_points: compartment_corner_points,
                })
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ConvertTanksToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertTanksToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
