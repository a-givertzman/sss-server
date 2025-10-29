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
    algorithm::
        eval::{
            import_3d_model_ctx::Import3DModelCtx, 
            Zg
        }
    , 
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::
        ContextWrite
    
};
///
/// Преобразование координат 3D модели 
/// в тип данных TriMesh
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
    /// Преобразование баттокса
    fn convert_buttocks(&self, buttocks: Vec<(f64,f64)>) -> Vec<TriMesh> {
        let mut result: Vec<TriMesh> = Vec::new();
        for coords in buttocks {
            let mut vertices = Vec::new();
            while vertices.len() <= 3 {
                vertices.push(
                    Point::new(coords.1, 0.0, coords.0) // тк это баттокс, то `y` всегда равен нулю
                );
            }
            match TriMesh::new(vertices, vec![[0, 1, 2]]) {
                Ok(trimesh) => {
                    result.push(
                        trimesh
                    );
                },
                Err(err) => {
                    panic!("{}", &format!("{} | {}", self.dbg, err));
                },
            }
        }
        result
    }
    ///
    /// Преобразование поверхности наружной/надстройки
    fn convert_surface(&self, surface: IndexMap<String,Vec<(f64,f64)>>) -> Vec<TriMesh> {
        let mut result: Vec<TriMesh> = Vec::new();
        for frame in surface {
            for coords in frame.1 {
                let mut vertices = Vec::new();
                while vertices.len() <= 3 {
                    vertices.push(
                        Point::new(
                            frame.0.parse()
                            .expect("Error to convert String to f64")
                            , coords.1
                            , coords.0
                        )
                    );
                }
                match TriMesh::new(vertices, vec![[0, 1, 2]]) {
                    Ok(trimesh) => {
                        result.push(
                            trimesh
                        );
                    },
                    Err(err) => {
                        panic!("{}", &format!("{} | {}", self.dbg, err));
                    },
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