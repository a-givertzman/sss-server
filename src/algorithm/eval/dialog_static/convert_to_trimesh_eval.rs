use indexmap::IndexMap;
use parry3d_f64::math::Point;
use parry3d_f64::shape::TriMesh;
use sal_core::{
    dbg::Dbg, 
    error::Error
};
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
    prelude::ContextWrite
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
    ///
    /// Новый экземпляр [ConvertToTrimeshEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertToTrimeshEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
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
            vertices.push(Point::new(*x, 0.0, *z)); // y = 0 для баттокса
        }
        for i in 1..vertices.len().saturating_sub(1) {
            indices.push([0, i as u32, (i + 1) as u32]);
        }
        if indices.is_empty() {
            return None;
        }
        match TriMesh::new(vertices, indices) {
            Ok(trimesh) => {
                Some(trimesh)
            },
            Err(err) => {
                None
            }
        }
    }
    ///
    /// Преобразование поверхности
    fn convert_surface(&self, surface: IndexMap<String, Vec<(f64, f64)>>) -> Option<TriMesh> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut frames: Vec<(f64, Vec<Point<f64>>)> = Vec::new();
        for (frame_name, coords) in surface {
            if let Ok(x) = frame_name.parse::<f64>() {
                let frame_vertices: Vec<Point<f64>> = coords
                    .iter()
                    .map(|(z, y)| Point::new(x, *y, *z))
                    .collect();
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
        let mut frame_vertex_indices = Vec::new();
        for (_, frame_verts) in &frames {
            let start_index = vertices.len();
            vertices.extend_from_slice(frame_verts);
            let end_index = vertices.len();
            frame_vertex_indices.push(start_index..end_index);
        }
        for i in 0..frames.len() - 1 {
            let current_indices = &frame_vertex_indices[i];
            let next_indices = &frame_vertex_indices[i + 1];
            let current_len = current_indices.len();
            let next_len = next_indices.len();
            let min_len = current_len.min(next_len);
            for j in 0..min_len - 1 {
                let current_idx = current_indices.start + j;
                let next_idx = next_indices.start + j;
                indices.push([
                    current_idx as u32,
                    (current_idx + 1) as u32,
                    next_idx as u32
                ]);
                indices.push([
                    (current_idx + 1) as u32,
                    (next_idx + 1) as u32,
                    next_idx as u32
                ]);
            }
        }
        if vertices.is_empty() || indices.is_empty() {
            log::warn!("{} | No vertices or indices generated", self.dbg);
            return None;
        }
        match TriMesh::new(vertices, indices) {
            Ok(trimesh) => {
                log::info!("{} | Created 3D surface: {} vertices, {} triangles", 
                        self.dbg, trimesh.vertices().len(), trimesh.indices().len());
                Some(trimesh)
            },
            Err(err) => {
                log::error!("{} | Error creating 3D surface: {}", self.dbg, err);
                None
            }
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for ConvertToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let model_3d = ContextRead::<Import3DModelCtx>::read(&ctx).clone();
                let stern_block = self.convert_buttocks(model_3d.stern_block.coordinates.clone())
                    .into_iter().collect();
                let nasal_block = self.convert_buttocks(model_3d.nasal_block.coordinates.clone())
                    .into_iter().collect();
                let surface_outer_body = self.convert_surface(model_3d.surface_outer_body.coordinates.clone()).into_iter().collect();
                let surface_superstructure = self.convert_surface(model_3d.surface_superstructure.coordinates.clone()).into_iter().collect();
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
//
//
impl std::fmt::Debug for ConvertToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}