use nalgebra::Point3;
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
use crate::algorithm::entities::points_manipulation::PointsManipulations;
use crate::algorithm::eval::import_model::convert_surface_outer_to_trimesh::convert_surface_outer_to_trimesh_ctx::ConvertSurfaceOuterToTrimeshCtx;
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
pub struct ConvertSurfaceOuterToTrimeshEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ConvertSurfaceOuterToTrimeshEval {
    ///
    /// Новый экземпляр [ConvertSurfaceOuterToTrimeshEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ConvertSurfaceOuterToTrimeshEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
    /// 
    /// Алгоритм «умного» соединения (часто называемый Shortest Diagonal) 
    /// работает по принципу жадного выбора: на каждом шаге мы решаем, 
    /// какую вершину продвинуть вперед — на верхнем слое или на нижнем, 
    /// чтобы минимизировать длину «разреза».
    pub fn create_trimesh_from_layers(
        &self, 
        layers_raw: Vec<Vec<(f64, f64, f64)>>,
        reverse_flag: bool,
    ) -> Result<TriMesh, Error> {
        let error = Error::new(&self.dbg, "create_trimesh_from_layers");
        if layers_raw.len() < 2 {
            return Err(error.err("Недостаточно слоев для создания объема (минимум 2)"));
        }
        let total_vtx: usize = layers_raw.iter().map(|l| l.len()).sum();
        let mut all_vertices = Vec::with_capacity(total_vtx);
        let mut all_indices = Vec::with_capacity(total_vtx * 2); 
        let mut layer_offsets = Vec::with_capacity(layers_raw.len());
        for layer in layers_raw {
            layer_offsets.push(all_vertices.len() as u32);
            if reverse_flag {
                all_vertices.extend(layer.into_iter().map(|p| Point3::new(p.0, -p.1 + 0.1, p.2)));
            } else {
                all_vertices.extend(layer.into_iter().map(|p| Point3::new(p.0, p.1, p.2)));
            }
        }
        for i in 0..layer_offsets.len() - 1 {
            let off_a = layer_offsets[i];
            let off_b = layer_offsets[i + 1];
            let n = (off_b - off_a) as usize;
            let m = if i + 2 < layer_offsets.len() {
                (layer_offsets[i + 2] - off_b) as usize
            } else {
                (all_vertices.len() as u32 - off_b) as usize
            };
            if n == 0 || m == 0 { continue; }
            let mut curr_i = 0;
            let mut curr_j = 0;
            for _ in 0..(n + m) {
                let next_i = (curr_i + 1) % n;
                let next_j = (curr_j + 1) % m;
                let move_a = if curr_i < n && curr_j < m {
                    let d_a = (all_vertices[(off_a + next_i as u32) as usize] - all_vertices[(off_b + curr_j as u32) as usize]).norm_squared();
                    let d_b = (all_vertices[(off_b + next_j as u32) as usize] - all_vertices[(off_a + curr_i as u32) as usize]).norm_squared();
                    d_a < d_b
                } else {
                    curr_i < n
                };
                if move_a {
                    all_indices.push([
                        off_a + curr_i as u32,
                        off_a + next_i as u32,
                        off_b + (curr_j % m) as u32,
                    ]);
                    curr_i += 1;
                } else {
                    all_indices.push([
                        off_a + (curr_i % n) as u32,
                        off_b + next_j as u32,
                        off_b + (curr_j % m) as u32,
                    ]);
                    curr_j += 1;
                }
            }
        }
        // Закрытие первого торца
        let first_layer = all_vertices[0..layer_offsets[1] as usize].to_vec();
        BuildWall::eval(
            true, 
            0, 
            &mut all_indices, 
            &mut all_vertices, 
            &first_layer, 
            layer_offsets[1] as usize
        );
        // Закрытие последнего торца
        let last_off = *layer_offsets.last().unwrap() as usize;
        let last_layer = all_vertices[last_off..all_vertices.len() - 1].to_vec(); // -1 из-за предыдущей центроиды
        BuildWall::eval(
            false, 
            last_off as u32, 
            &mut all_indices, 
            &mut all_vertices, 
            &last_layer, 
            last_layer.len() 
        );
        TriMesh::new(all_vertices, all_indices)
            .map_err(|_| error.err("Ошибка валидации TriMesh (возможно, вырожденные треугольники)"))
    }
}
//
//
impl Eval<Zg, EvalResult> for ConvertSurfaceOuterToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let model_3d = ContextRead::<ImportModelInitialPointsCtx>::read(&ctx).clone();
                let mut result = None;
                match self.create_trimesh_from_layers(model_3d.surface_outer_body.coordinates.clone(), false) {
                    Ok(mut ship_model) => {
                        ship_model.set_flags(TriMeshFlags::all()).unwrap();
                        let ship_model_reverse = PointsManipulations::mirror_mesh_y(&ship_model);
                        match PointsManipulations::add_trimeshes(&ship_model, &ship_model_reverse) {
                            Ok(add_res) => {
                                result = Some(add_res);
                            },
                            Err(_) => todo!(),
                        }
                    },
                    Err(e) => panic!("Error to create ship model: {}", e ),
                };
                ctx.write(
                    ConvertSurfaceOuterToTrimeshCtx {
                        result,
                    }
                )
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ConvertSurfaceOuterToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertSurfaceOuterToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}