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
    fn convert_buttocks(&self, buttocks: Vec<(f64, f64)>) -> Vec<TriMesh> {
        let mut result: Vec<TriMesh> = Vec::new();
        for i in 0..buttocks.len().saturating_sub(2) {
            let p1 = buttocks[i];
            let p2 = buttocks[i + 1];
            let p3 = buttocks[i + 2];
            let vertices = vec![
                Point::new(p1.1, 0.0, p1.0), // y = 0 для баттокса
                Point::new(p2.1, 0.0, p2.0),
                Point::new(p3.1, 0.0, p3.0),
            ];
            match TriMesh::new(vertices, vec![[0, 1, 2]]) {
                Ok(trimesh) => result.push(trimesh),
                Err(err) => panic!("{} | {}", self.dbg, err),
            }
        }
        result
    }
    ///
    /// Преобразование поверхности наружной/надстройки
    fn convert_surface(&self, surface: IndexMap<String, Vec<(f64, f64)>>) -> Vec<TriMesh> {
        let mut result: Vec<TriMesh> = Vec::new();
        let mut frames: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();
        for (frame_name, coords) in surface {
            if let Ok(frame_x) = frame_name.parse::<f64>() {
                frames.push((frame_x, coords));
            } else {
                log::warn!("{} | Cannot parse frame name: {}", self.dbg, frame_name);
            }
        }
        frames.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for i in 0..frames.len().saturating_sub(1) {
            let current_frame = &frames[i];
            let next_frame = &frames[i + 1];
            for j in 0..current_frame.1.len().saturating_sub(1) {
                if j < next_frame.1.len().saturating_sub(1) {
                    let p1 = current_frame.1[j];
                    let p2 = current_frame.1[j + 1];
                    let p3 = next_frame.1[j];
                    let p4 = next_frame.1[j + 1];
                    let vertices1 = vec![
                        Point::new(current_frame.0, p1.1, p1.0),
                        Point::new(current_frame.0, p2.1, p2.0),
                        Point::new(next_frame.0, p3.1, p3.0),
                    ];
                    let vertices2 = vec![
                        Point::new(current_frame.0, p2.1, p2.0),
                        Point::new(next_frame.0, p4.1, p4.0),
                        Point::new(next_frame.0, p3.1, p3.0),
                    ];
                    
                    if let Ok(trimesh1) = TriMesh::new(vertices1, vec![[0, 1, 2]]) {
                        result.push(trimesh1);
                    }
                    
                    if let Ok(trimesh2) = TriMesh::new(vertices2, vec![[0, 1, 2]]) {
                        result.push(trimesh2);
                    }
                }
            }
        }
        result
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
                let stern_block = self.convert_buttocks(model_3d.stern_block.coordinates.clone());
                let nasal_block = self.convert_buttocks(model_3d.nasal_block.coordinates.clone());
                let surface_outer_body = self.convert_surface(model_3d.surface_outer_body.coordinates.clone());
                let surface_superstructure = self.convert_surface(model_3d.surface_superstructure.coordinates.clone());
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