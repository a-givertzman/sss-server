use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use indexmap::IndexMap;
use multimap::MultiMap;
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
            vertices.push(Point::new(*x, 0.0, *z)); // y = 0
        }
        for i in 1..vertices.len().saturating_sub(1) {
            indices.push([0, i as u32, (i + 1) as u32]);
        }
        TriMesh::new(vertices, indices).ok()
    }
    fn save_points_to_txt(points: &[Point<f64>], path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for p in points {
            writeln!(writer, "{:.6} {:.6} {:.6}", p.x, p.y, p.z)?;
        }
        Ok(())
    }
    ///
    /// Проверяет угол между двумя векторами в градусах
    fn angle_between(&self, point1: OPoint<f64, Const<3>>, point2: OPoint<f64, Const<3>>) -> f64 {
        let vec1 = Vector3::new(point1.x, point1.y, point1.z);
        let vec2 = Vector3::new(point2.x, point2.y, point2.z);
        vec1.angle(&vec2)
    }
    ///
    /// Интерполяция фрейма до одинакового количества точек
    fn resample_line(&self, points: &[Point<f64>], n: usize) -> Vec<Point<f64>> {
        if points.len() < 2 || n < 2 {
            return points.to_vec();
        }
        let mut lengths = vec![0.0];
        for i in 1..points.len() {
            let prev = points[i - 1];
            let curr = points[i];
            let d = ((curr.x - prev.x).powi(2)
                + (curr.y - prev.y).powi(2)
                + (curr.z - prev.z).powi(2))
                .sqrt();
            lengths.push(lengths[i - 1] + d);
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
            result.push(Point::new(
                p1.x + (p2.x - p1.x) * t,
                p1.y + (p2.y - p1.y) * t,
                p1.z + (p2.z - p1.z) * t,
            ));
        }
        result
    }
    ///
    /// Преобразование поверхности
    fn convert_surface(&self, surface: Vec<Vec<(f64,f64,f64)>>) -> Option<TriMesh> {
        let mut frames: Vec<(f64, Vec<Point<f64>>)> = Vec::new();
        for vertices in &surface {
            if vertices.is_empty() {
                continue;
            }
            let frame = vertices[0].0;
            let points: Vec<Point<f64>> = vertices
                .iter()
                .map(|&(_, z, y)| Point::new(frame, y, z)) // x = frame, y = y, z = z
                .collect();
            frames.push((frame, points));
        }
        frames.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let all_points: Vec<Point<f64>> = frames.iter().flat_map(|(_, v)| v.clone()).collect();
        if let Err(e) = Self::save_points_to_txt(&all_points, "src\\tests\\unit\\algorithm\\dialog_static\\output_files\\resampled_points.txt") {
            log::error!("Failed to save points: {}", e);
        }
        for (_, verts) in frames.iter_mut() {
            *verts = self.resample_line(verts, 50);
        }

        let mut vertices: Vec<Point<f64>> = Vec::new();
        let mut indices: Vec<[u32; 3]> = Vec::new();
        for i in 0..frames.len() - 1 {
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
                    Some(triangle) => {
                        indices.push([base_index, base_index + 1, base_index + 2]);
                        match Triangle::new(p_p2, c_p2, c_p1).normal() {
                            Some(triangle) => {
                                indices.push([base_index + 1, base_index + 3, base_index + 2]);
                            },
                            None => {
                            },
                        }
                    },
                    None => {
                    },
                }
            }
            // if previous.len() == current.len() {
            //     println!("{:?}", previous.len());
            //     for j in 0..previous.len() - 1 {
            //         let p_p1 = previous[j];
            //         let p_p2 = previous[j + 1];
            //         let c_p1 = current[j];
            //         let c_p2 = current[j + 1];
            //         let base_index = vertices.len() as u32;
            //         vertices.push(p_p1);
            //         vertices.push(p_p2);
            //         vertices.push(c_p1);
            //         vertices.push(c_p2);
            //         indices.push([base_index, base_index + 1, base_index + 2]);
            //         indices.push([base_index + 1, base_index + 3, base_index + 2]);
            //         indices.push([base_index + 3, base_index, base_index + 2]);
            //     }
            // } else {
            //     println!("STOP");
            //     if previous.len() > current.len() {
            //         let new_frame_cur = self.resample_line(&current.clone(), previous.len());
            //         let new_frame_prev = self.resample_line(&previous.clone(), previous.len());
            //         // for i in 0..previous.len() {
            //         //     if i < current.len() {
            //         //         let p_p1 = previous[i];
            //         //         let c_p1 = current[i];
            //         //         let angle_between = self.angle_between(p_p1, c_p1).to_degrees();
            //         //         println!("angle between {} {}:  {}", p_p1, c_p1, angle_between);
            //         //     }
            //         // }
            //         // let tmp = [new_frame_cur.clone(), new_frame_prev.to_vec()].concat();
            //         // if let Err(e) = Self::save_points_to_txt(&tmp, "src\\tests\\unit\\algorithm\\dialog_static\\output_files\\resampled_points.txt") {
            //         //     log::error!("Failed to save points: {}", e);
            //         // }
            //         for j in 0..previous.len() - 1 {
            //             let p_p1 = new_frame_prev[j];
            //             let p_p2 = new_frame_prev[j + 1];
            //             let c_p1 = new_frame_cur[j];
            //             let c_p2 = new_frame_cur[j + 1];
            //             let base_index = vertices.len() as u32;
            //             vertices.push(p_p1);
            //             vertices.push(p_p2);
            //             vertices.push(c_p1);
            //             vertices.push(c_p2);
            //             indices.push([base_index, base_index + 1, base_index + 2]);
            //             indices.push([base_index + 1, base_index + 3, base_index + 2]);
            //             indices.push([base_index + 3, base_index, base_index + 2]);
            //         }
            //     }
            // }
        }
        match TriMesh::new(vertices, indices) {
            Ok(trimesh) => Some(trimesh),
            Err(err) => {
                log::error!("Failed to create TriMesh: {}", err);
                None
            },
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
