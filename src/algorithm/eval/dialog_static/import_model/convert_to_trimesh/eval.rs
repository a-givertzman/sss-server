use nalgebra::{Point3, Vector3};
use parry3d_f64::shape::{
    TriMesh, 
    TriMeshFlags
};
use sal_core::{
    dbg::Dbg, 
    error::Error
};
use crate::algorithm::context::context_access::ContextRead;
use poisson_reconstruction::PoissonReconstruction;
use crate::algorithm::eval::import_model::convert_to_trimesh::ctx::ConvertToTrimeshCtx;
use crate::algorithm::eval::import_model::import_model_initial_points::import_model_initial_points_ctx::ImportModelInitialPointsCtx;
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
        Self { dbg, ctx: Box::new(ctx) }
    } 
}
//
//
impl Eval<Zg, EvalResult> for ConvertToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let model_3d = ContextRead::<ImportModelInitialPointsCtx>::read(&ctx).clone();
                let mut points: Vec<Point3<f64>> = Vec::new();
                let mut normals: Vec<Point3<f64>> = Vec::new();

                let mut compute = |
                        new_frame: Vec<(f64, f64, f64)>
                | {
                    let mut new_frame: Vec<_> = new_frame.into_iter().map(|v| Point3::new(v.0, v.1, v.2)).collect();
                    let med_z: f64 = new_frame.iter().map(|v| v.z).sum::<f64>()/(new_frame.len() as f64);
                    let mut new_normals = new_frame.iter().map(|&v| Point3::new(0., v.y, v.z - med_z)).collect();
                    normals.append(&mut new_normals);
                    points.append(&mut new_frame);
                };

                compute(model_3d.bow_block);
                model_3d.surface_outer_body.coordinates.iter().for_each(|v| compute(v.to_vec()));
                compute(model_3d.stern_block);                

                let normals: Vec<Vector3<f64>> = normals
                .into_iter()
                .map(|p| p.coords)
                .collect();

                let poisson = PoissonReconstruction::from_points_and_normals(
                    &points, &normals, 1.0, 4, 5, 10
                );

                let mesh_buffers = poisson.reconstruct_mesh_buffers(); 

                let indicies = mesh_buffers.indices().chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();

                let ship_model = match TriMesh::new(mesh_buffers.vertices().into(), indicies) {
                    Ok(mut ship_model) => {
                        ship_model.set_flags(TriMeshFlags::all()).unwrap();
                        Some(ship_model)
                    },
                    Err(e) => panic!("Error to create ship model: {}", e ),
                };
                ctx.write(
                    ConvertToTrimeshCtx {
                        result: ship_model,
                    }
                )
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