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
use crate::algorithm::eval::convert_to_trimesh_ctx::ConvertToTrimeshCtx;
use crate::{
    algorithm::eval::{
        import_3d_model_ctx::Import3DModelCtx,
        Zg
    },
    kernel::{
        eval::Eval,
        types::eval_result::EvalResult
    },
    prelude::ContextWrite,
};
///
/// Преобразование координат 3D модели в тип данных TriMesh
pub struct ConvertToTrimeshEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ConvertToTrimeshEval {
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertToTrimeshEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
    ///
    /// Преобразование баттокса (корма/нос)
    fn convert_buttocks(&self, buttocks: Vec<(f64, f64)>) -> Option<TriMesh> {
        if buttocks.len() < 3 {
            log::warn!("{} | Not enough points for buttocks: {}", self.dbg, buttocks.len());
            return None;
        }
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for (z, x) in &buttocks {
            vertices.push(Point::new(*x, 0.0, *z));
        }
        for i in 1..vertices.len().saturating_sub(1) {
            indices.push([0, i as u32, (i + 1) as u32]);
        }
        TriMesh::new(vertices, indices).ok()
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
            let p1 = points[idx];
            let p2 = points[idx + 1];
            if idx + 2 < points.len() {
                let a = points[idx];
                let b = points[idx + 1];
                let c = points[idx + 2];
                let ba = Vector3::new(a.x - b.x, a.y - b.y, a.z - b.z);
                let bc = Vector3::new(c.x - b.x, c.y - b.y, c.z - b.z);
                let dot = ba.dot(&bc);
                let eps = 1e-6;
                if dot.abs() < eps {
                    result.push(b.clone());
                }
            }
            let interpolated_point = Point::new(
                p1.x + (p2.x - p1.x) * t,
                p1.y + (p2.y - p1.y) * t,
                p1.z + (p2.z - p1.z) * t,
            );
            result.push(interpolated_point);
        }
        result
    }
    ///
    /// Близость точек
    fn point_diff(&self, p1: Point<f64>, p2: Point<f64>, epsilon: f64) -> bool {
        (p1.x - p2.x).abs() <= epsilon && (p1.y - p2.y).abs() <= epsilon && (p1.z - p2.z).abs() <= epsilon
    }
    ///
    /// Преобразование поверхности
    fn convert_surface(&self, surface: Vec<Vec<(f64,f64,f64)>>) -> Option<TriMesh> {
        let mut frames: Vec<(f64, Vec<Point<f64>>)> = Vec::new();
        let mut same_frames: Vec<(f64, Vec<Point<f64>>)> = Vec::new();
        for vertices in &surface {
            if vertices.is_empty() {
                continue;
            }
            let frame = vertices[0].0;
            let points: Vec<Point<f64>> = vertices
                .iter()
                .map(|&(_, z, y)| Point::new(frame, y, z))
                .collect();
            if frames.len() > 0 {
                if frame == frames.last().unwrap().0 {
                    let last_frame = frames.last().unwrap();
                    same_frames.push((frame, points.clone()));
                    same_frames.push((frame, last_frame.1.clone()));
                }
            }
            frames.push((frame, points));
        }
        frames.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let all_points: Vec<OPoint<f64, Const<3>>> = frames.iter()
        .flat_map(|(_, points)| points.iter())
        .cloned()
        .collect();
        if let Err(e) = Self::save_points_to_txt(&all_points, "src\\tests\\unit\\algorithm\\dialog_static\\output_files\\sampled_points.txt") {
            log::error!("Failed to save points: {}", e);
        }
        let target_points = 300;
        for (_, verts) in frames.iter_mut() {
            *verts = self.resample_line(verts, target_points);
        }
        // let all_points: Vec<OPoint<f64, Const<3>>> = frames.iter()
        // .flat_map(|(_, points)| points.iter())
        // .cloned()
        // .collect();
        // if let Err(e) = Self::save_points_to_txt(&all_points, "src\\tests\\unit\\algorithm\\dialog_static\\output_files\\resampled_points.txt") {
        //     log::error!("Failed to save points: {}", e);
        // }
        let mut vertices: Vec<Point<f64>> = Vec::new();
        let mut indices: Vec<[u32; 3]> = Vec::new();
        for i in 0..frames.len() - 1 {
            if frames[i].0 == frames[i + 1].0 {
                // let prev_index = same_frames
                //     .iter()
                //     .position(|f| f.0 == frames[i].0)
                //     .unwrap();
                // let orig_prev_frame = same_frames[prev_index].clone();
                // let orig_cur_frame = same_frames[prev_index + 1].clone();
                // let mut previous = Vec::new();
                // let mut current = Vec::new();
                // for i in 0..orig_cur_frame.1.len() {
                //     if orig_cur_frame.1[i] != orig_prev_frame.1[i] {
                //         for j in (i-1)..orig_prev_frame.1.len() {
                //             previous.push(orig_prev_frame.1[j]);
                //         }
                //         for j in (i-1)..orig_cur_frame.1.len() {
                //             current.push(orig_cur_frame.1[j]);
                //         }
                //     }
                // }
                // previous = self.resample_line(&previous, target_points);
                // current = self.resample_line(&current, target_points);
                // let mut all_points: Vec<OPoint<f64, Const<3>>> = Vec::new();
                // all_points.extend(previous.clone());
                // all_points.extend(current.clone());
                // if let Err(e) = Self::save_points_to_txt(&all_points, "src\\tests\\unit\\algorithm\\dialog_static\\output_files\\resampled_points.txt") {
                //     log::error!("Failed to save points: {}", e);
                // }
                // for j in 0..previous.len() - 1 {
                //     let p_p1 = previous[j];
                //     let p_p2 = previous[j + 1];
                //     let c_p1 = current[j];
                //     let c_p2 = current[j + 1];
                //     let base_index = vertices.len() as u32;
                //     vertices.push(p_p1);
                //     vertices.push(p_p2);
                //     vertices.push(c_p1);
                //     vertices.push(c_p2);
                //     match Triangle::new(p_p1, p_p2, c_p1).normal() {
                //         Some(_) => {
                //             indices.push([base_index, base_index + 1, base_index + 2]);
                //             match Triangle::new(p_p2, c_p2, c_p1).normal() {
                //                 Some(_) => {
                //                     indices.push([base_index + 1, base_index + 3, base_index + 2]);
                //                 },
                //                 None => {},
                //             }
                //         },
                //         None => {},
                //     }
                // }
            } else {
                let previous = &frames[i].1;
                let current = &frames[i + 1].1;
                for j in 0..previous.len() - 1 {
                    let p_p1 = previous[j];
                    let p_p2 = previous[j + 1];
                    let c_p1 = current[j];
                    let c_p2 = current[j + 1];
                    let base_index = vertices.len() as u32;
                    vertices.push(p_p1);
                    vertices.push(p_p2);
                    vertices.push(c_p1);
                    vertices.push(c_p2);
                    match Triangle::new(p_p1, p_p2, c_p1).normal() {
                        Some(_) => {
                            indices.push([base_index, base_index + 1, base_index + 2]);
                            match Triangle::new(p_p2, c_p2, c_p1).normal() {
                                Some(_) => {
                                    indices.push([base_index + 1, base_index + 3, base_index + 2]);
                                },
                                None => {},
                            }
                        },
                        None => {},
                    }
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

impl Eval<Zg, EvalResult> for ConvertToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let model_3d = ContextRead::<Import3DModelCtx>::read(&ctx).clone();
                // let stern_block = self
                //     .convert_buttocks(model_3d.stern_block.coordinates.clone())
                //     .into_iter()
                //     .collect();
                // let nasal_block = self
                //     .convert_buttocks(model_3d.nasal_block.coordinates.clone())
                //     .into_iter()
                //     .collect();
                let surface_outer_body = self
                    .convert_surface(model_3d.surface_outer_body.coordinates.clone());
                // let surface_superstructure = self
                //     .convert_surface(model_3d.surface_superstructure.coordinates.clone())
                //     .into_iter()
                //     .collect();
                ctx.write(ConvertToTrimeshCtx {
                    stern_block: None,
                    nasal_block: None,
                    surface_outer_body,
                    surface_superstructure: None,
                })
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}

impl std::fmt::Debug for ConvertToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
