use nalgebra::{
    Const, 
    OPoint
};
use parry3d_f64::shape::{
    TriMesh, 
    TriMeshFlags
};
use sal_core::{
    dbg::Dbg, 
    error::Error
};
use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::entities::build_wall::BuildWall;
use crate::algorithm::entities::points_manipulation::{IntoPoints, PointsManipulations};
use crate::algorithm::entities::resample_line::resample_line;
use crate::algorithm::eval::import_model::convert_surface_superstructure_to_trimesh::convert_surface_superstructure_to_trimesh_ctx::ConvertSurfaceSuperStructureToTrimeshCtx;
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
pub struct ConvertSurfaceSuperStructureToTrimeshEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ConvertSurfaceSuperStructureToTrimeshEval {
    ///
    /// Новый экземпляр [ConvertSurfaceSuperStructureToTrimeshEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertSurfaceSuperStructureToTrimeshEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
    ///
    /// Создание и индексирование вершин
    /// - `tanks_3d` - набор блоков координат отсеков
    /// - `target_points` - кол-во точек на шпангоут для интерполяции
    fn convert_surface_superstructure(&self, tanks_3d: ImportModelInitialPointsCtx, mut target_points: usize) -> Option<TriMesh> {
        let mut all_vertices: Vec<OPoint<f64, Const<3>>> = Vec::new();
        let mut all_indices: Vec<[u32; 3]> = Vec::new();
        let mut walls_diff_y = Vec::new();
        let mut prev_points: Option<Vec<OPoint<f64, Const<3>>>> = None;
        let mut first = Vec::new();
        for i in 0..tanks_3d.surface_superstructure.len() {
            let frame: &Vec<(f64, f64, f64)> = &tanks_3d.surface_superstructure[i];
            let points_vec = frame.clone().into_points();
            let points: Vec<OPoint<f64, Const<3>>> = resample_line(&points_vec, target_points);
            if i == 0 {
                first = points.clone();
            }
            let opoints: Vec<_> = points.iter().map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z)).collect();
            if all_vertices.len() == 0 {
                all_vertices.extend_from_slice(&opoints);
            } else {
                let prev_x = tanks_3d.surface_superstructure[i-1][0].0;
                let curr_x = frame[0].0;
                if prev_x == curr_x {
                    let mut gapped_points: Vec<OPoint<f64, Const<3>>> = opoints.iter().map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z)).collect();
                    if let Some(prev) = &prev_points {
                        let ress = PointsManipulations::make_frame(&points_vec, &tanks_3d.surface_superstructure[i - 1].clone().into_points(), &gapped_points, prev);
                        gapped_points = ress.1;
                        let mut wall_vert = Vec::new();
                        wall_vert.extend_from_slice(&ress.0.to_vec());
                        let mut wall_ind = Vec::new();
                        BuildWall::eval(true, 0 as u32, &mut wall_ind, &mut wall_vert, &ress.0.to_vec(),  ress.0.len() - 1); // нос
                        walls_diff_y.push(TriMesh::new(wall_vert, wall_ind).expect("Error to build wall"));
                    }
                    all_vertices.extend_from_slice(&gapped_points);
                    target_points = gapped_points.len();
                    prev_points = Some(gapped_points);
                    continue;
                } else {
                    if let Some(prev) = &prev_points {
                        PointsManipulations::connect_points_for_ship(
                            &mut all_vertices,
                            &mut all_indices,
                            prev,
                            &opoints,
                            true,
                            false,
                        );
                    }
                }
            }
            prev_points = Some(opoints);
        }
        BuildWall::eval(false, 0, &mut all_indices, &mut all_vertices, &first.to_vec(), first.len()); // корма
        let last_start = all_vertices.len() - target_points - 1;
        if let Some(last) = all_vertices.clone().get(last_start..) {
            BuildWall::eval(true, last_start as u32, &mut all_indices, &mut all_vertices, &last.to_vec(),  last.len() - 1); // нос
        }
        match TriMesh::new(all_vertices, all_indices) {
            Ok(mut ship_model) => {
                for wall in walls_diff_y {
                    ship_model.append(&wall);
                }
                let _ = ship_model.set_flags(TriMeshFlags::MERGE_DUPLICATE_VERTICES);
                let _ = ship_model.set_flags(TriMeshFlags::DELETE_DUPLICATE_TRIANGLES);
                let _ = ship_model.set_flags(TriMeshFlags::DELETE_DEGENERATE_TRIANGLES);
                let _ = ship_model.set_flags(TriMeshFlags::DELETE_BAD_TOPOLOGY_TRIANGLES);
                let _ = ship_model.set_flags(TriMeshFlags::FIX_INTERNAL_EDGES);
                let _ = ship_model.set_flags(TriMeshFlags::ORIENTED);
                Some(ship_model)
            },
            Err(e) => panic!("Error to create ship model: {}",e ),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for ConvertSurfaceSuperStructureToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let target_points = 200 ; // кол-во точек для интерполяции
                let model_3d = ContextRead::<ImportModelInitialPointsCtx>::read(&ctx).clone();
                let surface_outer_body = self
                    .convert_surface_superstructure(model_3d, target_points);
                ctx.write(
                    ConvertSurfaceSuperStructureToTrimeshCtx {
                        result: surface_outer_body,
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ConvertSurfaceSuperStructureToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertSurfaceSuperStructureToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}