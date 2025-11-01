use indexmap::IndexMap;
use parry3d_f64::math::Point;
use parry3d_f64::shape::TriMesh;
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
    fn convert_surface(&self, surface: IndexMap<String, Vec<(f64, f64)>>) -> Option<TriMesh> {
        let mut frames: Vec<(f64, Vec<Point<f64>>)> = Vec::new();
        for (frame_name, coords) in surface {
            if let Ok(x) = frame_name.parse::<f64>() {
                let mut frame_vertices: Vec<Point<f64>> = coords
                    .iter()
                    .map(|(z, y)| Point::new(x, *y, *z))
                    .collect();
                frame_vertices.retain(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
                if frame_vertices.len() >= 2 {
                    frames.push((x, frame_vertices));
                }
            }
        }
        frames.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        if frames.len() < 2 {
            log::warn!("{} | Need at least 2 frames for 3D surface", self.dbg);
            return None;
        }
        let max_len = frames.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
        for (_, verts) in &mut frames {
            *verts = self.resample_line(verts, max_len);
        }
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut frame_indices = Vec::new();
        for (_, verts) in &frames {
            let start = vertices.len();
            vertices.extend_from_slice(verts);
            frame_indices.push(start..vertices.len());
        }
        for i in 0..frames.len() - 1 {
            let a = &frame_indices[i];
            let b = &frame_indices[i + 1];
            for j in 0..(a.len() - 1) {
                let a0 = a.start + j;
                let a1 = a.start + j + 1;
                let b0 = b.start + j;
                let b1 = b.start + j + 1;

                indices.push([a0 as u32, a1 as u32, b0 as u32]);
                indices.push([a1 as u32, b1 as u32, b0 as u32]);
            }
        }
        if vertices.is_empty() || indices.is_empty() {
            log::warn!("{} | No vertices or indices generated", self.dbg);
            return None;
        }
        match TriMesh::new(vertices, indices) {
            Ok(trimesh) => {
                log::info!(
                    "{} | Created 3D surface: {} vertices, {} triangles",
                    self.dbg,
                    trimesh.vertices().len(),
                    trimesh.indices().len()
                );
                Some(trimesh)
            }
            Err(err) => {
                log::error!("{} | Error creating 3D surface: {}", self.dbg, err);
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
                let stern_block = self
                    .convert_buttocks(model_3d.stern_block.coordinates.clone())
                    .into_iter()
                    .collect();
                let nasal_block = self
                    .convert_buttocks(model_3d.nasal_block.coordinates.clone())
                    .into_iter()
                    .collect();
                let surface_outer_body = self
                    .convert_surface(model_3d.surface_outer_body.coordinates.clone())
                    .into_iter()
                    .collect();
                let surface_superstructure = self
                    .convert_surface(model_3d.surface_superstructure.coordinates.clone())
                    .into_iter()
                    .collect();
                ctx.write(ConvertToTrimeshCtx {
                    stern_block,
                    nasal_block,
                    surface_outer_body,
                    surface_superstructure,
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
