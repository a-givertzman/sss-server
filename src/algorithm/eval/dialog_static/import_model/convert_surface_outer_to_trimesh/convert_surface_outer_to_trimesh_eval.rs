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
    /// Соединение шпангоутов ГП (главной палубы)
    /// - `main_deck` - набор шпангоутов ГП
    /// - `target_points` - кол-во точек в шпангоутах
    fn connect_main_deck(
        &self,
        main_deck: Vec<Vec<OPoint<f64, Const<3>>>>,
        target_points: usize
    ) -> TriMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut prev_points: Option<Vec<OPoint<f64, Const<3>>>> = None;
        for i in 0..main_deck.len() {
            let frame = &main_deck[i];
            let opoints: Vec<OPoint<f64, Const<3>>> = resample_line(&frame, target_points);
            if i == 0 {
                vertices.extend_from_slice(&opoints);
            } else {
                let prev_x = main_deck[i-1][0].x;
                let curr_x = frame[0].x;
                if prev_x == curr_x {
                    if let Some(prev) = &prev_points {
                        PointsManipulations::connect_points(
                            &mut vertices,
                            &mut indices,
                            prev,
                            &opoints,
                            true,
                            false,
                        );
                    }
                    prev_points = Some(opoints);
                    continue;
                } else {
                    if let Some(prev) = &prev_points {
                        PointsManipulations::connect_points(
                            &mut vertices,
                            &mut indices,
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
        match TriMesh::new(vertices, indices) {
            Ok(trimesh) => trimesh,
            Err(e) => panic!("Error to connect main deck: {:?}", e),
        }
    }
    ///
    /// Создание и индексирование вершин
    /// - `tanks_3d` - набор блоков координат отсеков
    /// - `target_points` - кол-во точек на шпангоут для интерполяции
    fn convert_surface_outer(&self, tanks_3d: ImportModelInitialPointsCtx, mut target_points: usize) -> Option<TriMesh> {
        let mut all_vertices: Vec<OPoint<f64, Const<3>>> = Vec::new();
        let mut all_indices: Vec<[u32; 3]> = Vec::new();
        let mut walls_diff_y = Vec::new();
        let mut mirror_mask: Vec<bool> = Vec::new();
        let mut prev_points: Option<Vec<OPoint<f64, Const<3>>>> = None;
        let mut main_deck = Vec::new();
        let mut flag_last_main_deck = false; // мы сейчас внутри диапазона main_deck
        let mut last_lower_gapped: Option<Vec<OPoint<f64, Const<3>>>> = None;
        let mut first = Vec::new();
        let mut last = Vec::new();
        for i in 0..tanks_3d.surface_outer_body.coordinates.len() {
            let frame: &Vec<(f64, f64, f64)> = &tanks_3d.surface_outer_body.coordinates[i];
            let points_vec = frame.clone().into_points();
            let is_main_deck = tanks_3d.surface_outer_body.main_deck.contains(&i);
            let points: Vec<OPoint<f64, Const<3>>> = if is_main_deck {
                let (upper, lower) = PointsManipulations::split_and_resample(&points_vec, target_points);
                let lower_gapped: Vec<OPoint<f64, Const<3>>> = lower.into_iter().map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z)).collect();
                if !flag_last_main_deck {
                    if let Some(prev) = prev_points.as_ref() {
                        let prev_mirror = PointsManipulations::mirror_points(prev.clone());
                        let prev_mirror_ordered = PointsManipulations::order_points_like(&lower_gapped, &prev_mirror);
                        main_deck.push(prev_mirror_ordered);
                    }
                    flag_last_main_deck = true;
                }
                last_lower_gapped = Some(lower_gapped.clone());
                main_deck.push(lower_gapped);   
                upper.into_iter().map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z)).collect()
            } else {
                resample_line(&points_vec, target_points)
            };
            if i == 0 {
                first = points.clone();
            } else if i == tanks_3d.surface_outer_body.coordinates.len() - 1 {
                last = points.clone();
            }
            let opoints: Vec<_> = points.iter().map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z)).collect();
            // первый фрейм после main_deck (переход main_deck: true -> false)
            if !is_main_deck && flag_last_main_deck {
                if let Some(reference) = last_lower_gapped.as_ref() {
                    let after_mirror = PointsManipulations::mirror_points(opoints.clone());
                    let after_mirror_ordered = PointsManipulations::order_points_like(reference, &after_mirror);
                    main_deck.push(after_mirror_ordered);
                } else {
                    main_deck.push(PointsManipulations::mirror_points(opoints.clone()));
                }
                flag_last_main_deck = false;
            }
            if all_vertices.len() == 0 {
                all_vertices.extend_from_slice(&opoints);
                mirror_mask.extend(std::iter::repeat(!is_main_deck).take(target_points));
            } else {
                let prev_x = tanks_3d.surface_outer_body.coordinates[i-1][0].0;
                let curr_x = frame[0].0;
                if prev_x == curr_x {
                    let mut gapped_points: Vec<OPoint<f64, Const<3>>> = opoints.iter().map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z)).collect();
                    if let Some(prev) = &prev_points {
                        let ress = PointsManipulations::make_frame(&points_vec, &tanks_3d.surface_outer_body.coordinates[i - 1].clone().into_points(), &gapped_points, prev);
                        gapped_points = ress.1;
                        let mut wall_vert = Vec::new();
                        wall_vert.extend_from_slice(&ress.0.to_vec());
                        let mut wall_ind = Vec::new();
                        BuildWall::eval(true, 0 as u32, &mut wall_ind, &mut wall_vert, &ress.0.to_vec(),  ress.0.len() - 1);
                        PointsManipulations::mirror_vert_ind(&mut wall_vert, &mut wall_ind, &[]);
                        walls_diff_y.push(TriMesh::new(wall_vert, wall_ind).expect("Error to build wall"));
                    }
                    mirror_mask.extend(std::iter::repeat(!is_main_deck).take(target_points));
                    all_vertices.extend_from_slice(&gapped_points);
                    target_points = gapped_points.len();
                    prev_points = Some(gapped_points);
                    continue;
                } else {
                    if let Some(prev) = &prev_points {
                        PointsManipulations::connect_points(
                            &mut all_vertices,
                            &mut all_indices,
                            prev,
                            &opoints,
                            true,
                            false,
                        );
                    }
                    mirror_mask.extend(std::iter::repeat(!is_main_deck).take(target_points));
                }
            }
            prev_points = Some(opoints);
        }
        BuildWall::eval(false, 0, &mut all_indices, &mut all_vertices, &first.to_vec(), first.len()); // корма
        BuildWall::eval(true, (all_vertices.len() - last.len()) as u32, &mut all_indices, &mut all_vertices, &last.to_vec(),  last.len() - 1); // нос
        PointsManipulations::mirror_vert_ind(&mut all_vertices, &mut all_indices, &mirror_mask);
        match TriMesh::new(all_vertices, all_indices) {
            Ok(mut ship_model) => {
                for wall in walls_diff_y {
                    ship_model.append(&wall);
                }
                if !main_deck.is_empty() {
                    let main_deck = self.connect_main_deck(main_deck, target_points);
                    ship_model.append(&main_deck);

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
impl Eval<Zg, EvalResult> for ConvertSurfaceOuterToTrimeshEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let target_points = 200 ; // кол-во точек для интерполяции
                let model_3d = ContextRead::<ImportModelInitialPointsCtx>::read(&ctx).clone();
                let surface_outer_body = self
                    .convert_surface_outer(model_3d, target_points);
                ctx.write(
                    ConvertSurfaceOuterToTrimeshCtx {
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
impl std::fmt::Debug for ConvertSurfaceOuterToTrimeshEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvertSurfaceOuterToTrimeshEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}