use nalgebra::{
    Const, 
    OPoint
};
use parry3d_f64::math::Point;
use parry3d_f64::shape::{
    TriMesh, 
};
use sal_core::{
    dbg::Dbg, 
    error::Error
};
use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::entities::build_wall::BuildWall;
use crate::algorithm::entities::points_manipulation::IntoPoints;
use crate::algorithm::eval::import_model::convert_diametrical_buttocks_to_trimesh::convert_diametrical_buttocks_to_trimesh_ctx::ConvertDiametricalButtocksToTrimeshCtx;
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
pub struct ConvertDiametricalButtocksToTrimeshEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ConvertDiametricalButtocksToTrimeshEval {
    ///
    /// Новый экземпляр [ConvertDiametricalButtocksToTrimeshEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertDiametricalButtocksToTrimeshEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
    ///
    /// Создание и индексирование вершин
    /// - `tanks_3d` - набор блоков координат отсеков
    /// - `target_points` - кол-во точек на шпангоут для интерполяции
    fn convert_diametrical_buttocks(&self, tanks_3d: ImportModelInitialPointsCtx) -> (Option<TriMesh>, Option<TriMesh>) {
        // stern buttocks
        let stern_points = (tanks_3d.stern_block).into_points();
        let mut stern_vertices = Vec::new();
        stern_vertices.extend_from_slice(&stern_points);
        let mut stern_indices = Vec::new();
        BuildWall::eval(false, 0, &mut stern_indices, &mut stern_vertices, &stern_points, stern_points.len());
        // bow buttocks
        let bow_points = (tanks_3d.bow_block).into_points();
        let mut bow_vertices = Vec::new();
        bow_vertices.extend_from_slice(&bow_points);
        let mut bow_indices = Vec::new();
        BuildWall::eval(false, 0, &mut bow_indices, &mut bow_vertices, &bow_points, bow_points.len());
        let mut stern_buttocks = None;
        let mut bow_buttocks = None;
        match TriMesh::new(stern_vertices, stern_indices) {
            Ok(res) => stern_buttocks = Some(res),
            Err(e) => log::error!("Error to create stern buttocks: {:?}", e),
        }
        match TriMesh::new(bow_vertices, bow_indices) {
            Ok(res) => bow_buttocks = Some(res),
            Err(e) => log::error!("Error to create bow buttocks: {:?}", e),
        }
        (stern_buttocks, bow_buttocks)
    }
}
//
//
impl Eval<Zg, EvalResult> for ConvertDiametricalButtocksToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let model_3d = ContextRead::<ImportModelInitialPointsCtx>::read(&ctx).clone();
                let (stern_buttocks, bow_buttocks) = self
                    .convert_diametrical_buttocks(model_3d);
                ctx.write(
                    ConvertDiametricalButtocksToTrimeshCtx {
                        stern_buttocks,
                        bow_buttocks,
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ConvertDiametricalButtocksToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertDiametricalButtocksToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}