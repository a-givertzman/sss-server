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
        let mut frames: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();
        for (frame_name, coords) in surface {
            if let Ok(frame_x) = frame_name.parse::<f64>() {
                if !coords.is_empty() {
                    frames.push((frame_x, coords));
                }
            } else {
                log::warn!("{} | Cannot parse frame name: {}", self.dbg, frame_name);
            }
        }
        
        if frames.is_empty() {
            return None;
        }
        frames.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let min_points_per_frame = frames.iter()
            .map(|(_, points)| points.len())
            .min()
            .unwrap_or(0);
        if min_points_per_frame < 2 {
            return None;
        }
        if frames.len() < 2 {
            return None;
        }
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        for (frame_x, points) in &frames {
            for i in 0..min_points_per_frame {
                let (z, y) = points[i];
                all_vertices.push(Point::new(*frame_x, y, z));
            }
        }
        for frame_idx in 0..frames.len() - 1 {
            for point_idx in 0..min_points_per_frame - 1 {
                let current_base = (frame_idx * min_points_per_frame) as u32;
                let next_base = ((frame_idx + 1) * min_points_per_frame) as u32;
                let i00 = current_base + point_idx as u32;
                let i01 = current_base + (point_idx + 1) as u32;
                let i10 = next_base + point_idx as u32;
                let i11 = next_base + (point_idx + 1) as u32;
                all_indices.push([i00, i01, i10]);
                all_indices.push([i01, i11, i10]);
            }
        }
        if all_indices.is_empty() {
            log::warn!("{} | No triangles created for surface", self.dbg);
            return None;
        }
        match TriMesh::new(all_vertices, all_indices) {
            Ok(trimesh) => {
                log::info!("{} | Created surface mesh: {} vertices, {} triangles", 
                    self.dbg, trimesh.vertices().len(), trimesh.triangles().len());
                Some(trimesh)
            },
            Err(err) => {
                log::error!("{} | Error creating surface mesh: {}", self.dbg, err);
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
                let surface_outer_body = self.convert_surface(model_3d.surface_outer_body.coordinates.clone())
                    .into_iter().collect();
                let surface_superstructure = self.convert_surface(model_3d.surface_superstructure.coordinates.clone())
                    .into_iter().collect();
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