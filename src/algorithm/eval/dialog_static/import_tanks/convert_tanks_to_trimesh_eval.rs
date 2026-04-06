use parry3d_f64::shape::{
    TriMesh, TriMeshFlags, 
};
use sal_core::{
    dbg::Dbg, 
    error::Error
};
use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::entities::points_manipulation::{
    IntoPoints, 
    PointsManipulations
};
use crate::algorithm::eval::import_model::convert_surface_outer_to_trimesh::convert_surface_outer_to_trimesh_ctx::ConvertSurfaceOuterToTrimeshCtx;
use crate::algorithm::eval::import_tanks::convert_tanks_to_trimesh_ctx::ConvertTanksToTrimeshCtx;
use crate::algorithm::eval::import_tanks::import_3d_tanks_ctx::Import3DTanksCtx;
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
pub struct ConvertTanksToTrimeshEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ConvertTanksToTrimeshEval {
    ///
    /// Новый экземпляр [ConvertTanksToTrimeshEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertTanksToTrimeshEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
    ///
    /// Создание и индексирование вершин
    /// - `tanks_3d` - набор блоков координат отсеков
    fn get_vertices_indeces(&self, tanks_3d: Import3DTanksCtx) -> Vec<TriMesh> {
        let mut result: Vec<TriMesh> = Vec::new();
        let mut last_id = 0.0;
        for comp_corner in tanks_3d.compartment_corner_points {
            let x1 = comp_corner.coordinates_x1.0;
            let x2 = comp_corner.coordinates_x2.0;
            let points_x1 = (x1, comp_corner.coordinates_x1.1.clone()).into_points();
            let points_x2 = (x2, comp_corner.coordinates_x2.1.clone()).into_points();
            let mut vertices = Vec::with_capacity(points_x1.len() + points_x2.len());
            let mut indices = Vec::with_capacity((points_x1.len() + points_x2.len()) * 2);
            if x1 < x2 {
                PointsManipulations::connect_points_for_tanks(&mut vertices, &mut indices, &points_x1, &points_x2, false);
            } else {
                PointsManipulations::connect_points_for_tanks(&mut vertices, &mut indices, &points_x1, &points_x2, true);
            }
            if result.len() == 0 {
                match TriMesh::new(vertices, indices) {
                    Ok(mut trimesh) => {
                        let _ = trimesh.set_flags(TriMeshFlags::all());
                        result.push(trimesh);
                    },
                    Err(e) => log::error!("Error to create TriMesh: {}", e),
                }
            } else {
                if last_id == comp_corner.id {
                    let last_figure = result.pop().unwrap();
                    let mut curr_figure = TriMesh::new(vertices, indices).expect("Error to create TriMesh");
                    let _ = curr_figure.set_flags(TriMeshFlags::all());
                    if x1 < x2 { 
                        match PointsManipulations::add_trimeshes(&last_figure, &curr_figure) {
                            Ok(add_res) => {
                                result.push(add_res);
                            },
                            Err(e) => log::error!("Error to add TriMeshes: {}", e),
                        }
                    } else { 
                        match PointsManipulations::substract_trimeshes(&last_figure, &curr_figure) {
                            Ok(sub_res) => {
                                result.push(sub_res);
                            },
                            Err(e) => log::error!("Error to substract TriMeshes: {}", e),
                        }
                    }
                } else {
                    if tanks_3d.compartment_id_to_reverse.contains(&(-last_id)) {
                        let last_figure = result.pop().unwrap();
                        match PointsManipulations::mirror_y(&last_figure) {
                            Ok(trimesh) => {
                                result.push(trimesh)
                            },
                            Err(e) => {
                                log::error!("Error to mirror TriMesh: {}", e);
                                result.push(last_figure);
                            }
                        }
                    }
                    match TriMesh::new(vertices, indices) {
                        Ok(mut trimesh) => {
                            let _ = trimesh.set_flags(TriMeshFlags::all());
                            result.push(trimesh);
                        },
                        Err(e) => log::error!("Error to create TriMesh: {}", e),
                    }
                }
            }
            last_id = comp_corner.id;
        }
        result
    }
    ///
    /// Подгон отсеков под модель корабля
    /// - `ship_model` - модель корабль
    /// - `tanks` - массив моделей отсеков
    fn substract_ship_model(&self, ship_model: TriMesh, tanks: Vec<TriMesh>) -> Option<Vec<TriMesh>> {
        let mut full_tanks = Vec::with_capacity(tanks.len());
        let ship_model_reverse = PointsManipulations::mirror_mesh_y(&ship_model);
        for tank in tanks {
            match PointsManipulations::intersection_trimeshes(&ship_model, &tank) {
                Ok(sub_res) => {
                    full_tanks.push(sub_res);
                },
                Err(e) => {
                    log::error!("Error to substract tank: {:?}", e);
                    full_tanks.push(tank);
                }
            }
        }
        Some(full_tanks)
    }
}
//
//
impl Eval<Zg, EvalResult> for ConvertTanksToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let tanks_3d = ContextRead::<Import3DTanksCtx>::read(&ctx).clone();
                let model_3d = ContextRead::<ConvertSurfaceOuterToTrimeshCtx>::read(&ctx).clone();
                let result = self.substract_ship_model(
                    model_3d.result.expect("Error to get result of `ConvertSurfaceOuterToTrimeshCtx`"), 
                    self.get_vertices_indeces(tanks_3d)
                );
                ctx.write(
                    ConvertTanksToTrimeshCtx {
                        tank: result,
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ConvertTanksToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertTanksToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}