use std::io::Write;
use std::path::PathBuf;

use boolmesh::prelude::{
    self, 
    Manifold
};
use csgrs::mesh::Mesh;
use nalgebra::{
    Const, 
    OPoint
};
use boolmesh::{
    self, 
    compute_boolean
};
use parry3d_f64::math::Point;
use parry3d_f64::shape::{
    TriMesh, 
    TriMeshFlags
};
use sal_core::{
    dbg::Dbg, 
    error::Error
};
use crate::algorithm::context::context_access::ContextRead;
use crate::algorithm::eval::import_model::convert_model_to_trimesh_ctx::ConvertModelToTrimeshCtx;
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
/// Write data to .stl file
pub fn write_stl(path: &PathBuf, mesh: &TriMesh) -> Result<(), Error> {
    let error = Error::new("Shape", "write_stl");
    let (result, empty_normals): (Vec<_>, Vec<_>) = mesh
        .triangles()
        .map(|t| (t.normal(), t))
        .partition(|(n, _)| n.is_some());
    if !empty_normals.is_empty() {
        return Err(error.err(format!("calculate normal error, path:{:?}", path)));
    }
    let triangles: Vec<_> = result
        .into_iter()
        .map(|(n, t)| {
            let n = n.unwrap();
            let normal = stl_io::Vector([n[0] as f32, n[1] as f32, n[2] as f32]);
            let vertices = [
                stl_io::Vector([t.a[0] as f32, t.a[1] as f32, t.a[2] as f32]),
                stl_io::Vector([t.b[0] as f32, t.b[1] as f32, t.b[2] as f32]),
                stl_io::Vector([t.c[0] as f32, t.c[1] as f32, t.c[2] as f32]),
            ];
            stl_io::Triangle { normal, vertices }
        })
        .collect();
    let mut binary_stl = Vec::<u8>::new();
    stl_io::write_stl(&mut binary_stl, triangles.iter())
        .map_err(|err| error.pass_with("stl_io::write_stl", err.to_string()))?;
    let mut buffer = std::fs::File::create(&path).map_err(|err| {
        error.pass_with(format!("File::create, path:{:?}", path), err.to_string())
    })?;
    buffer.write_all(&binary_stl).map_err(|err| {
        error.pass_with(
            format!("buffer.write_all, path:{:?}", path),
            err.to_string(),
        )
    })
}
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
    /// Преобразование координат в набор точек [Point]
    fn convert_to_points_vec(
        &self,
        x: f64,
        vec_z_y: Vec<(f64,f64)>, 
    ) -> Vec<Point<f64>>{
        let mut frame: Vec<Point<f64>> = Vec::new();
        for (y, z) in vec_z_y {
            let point = Point::new(x, -y, z);
            frame.push(point);
        }
        return frame;
    }
    ///
    /// Соединение точек из
    /// двух блоков координат
    /// - `vertices` - набор вершин 3D фигуры
    /// - `indices` - набор индексов вершин 3D фигуры
    /// - `points_x1` - первый блок точек для соединения
    /// - `points_x2` - второй блок точек для соединения
    fn connect_points(
        &self,
        vertices: &mut Vec<OPoint<f64, Const<3>>>,
        indices: &mut Vec<[u32; 3]>,
        points_x1: &[OPoint<f64, Const<3>>],
        points_x2: &[OPoint<f64, Const<3>>],
        reverse: bool,
    ) {
        assert!(points_x1.len() >= 3);
        assert_eq!(points_x1.len(), points_x2.len());
        let n = points_x1.len();
        let mut count_dublicats = 0;
        let base = vertices.len() as u32;
        for point in points_x1 {
            if !vertices.contains(point) {
                vertices.push(*point);
            } else {
                count_dublicats += 1;
            }
        }
        for point in points_x2 {
            if !vertices.contains(point) {
                vertices.push(*point);
            }
        }
        let x1 = base;
        let x2 = base + (n - count_dublicats) as u32;
        // боковые торцы
        for i in 0..(n - count_dublicats) {
            let next = (i + 1) % (n - count_dublicats);
            let a = x1 + i as u32;
            let b = x1 + next as u32;
            let c = x2 + next as u32;
            let d = x2 + i as u32;
            if reverse {
                indices.push([a, c, b]);
                indices.push([a, d, c]);
            } else {
                indices.push([a, b, c]);
                indices.push([a, c, d]);
            }
        }
        // стенка x1
        for i in 1..(n - count_dublicats) - 1 {
            if reverse {
                indices.push([
                    x1,
                    x1 + i as u32,
                    x1 + i as u32 + 1,
                ]);
            } else {
                indices.push([
                    x1,
                    x1 + i as u32 + 1,
                    x1 + i as u32,
                ]);
            }
        }
        // стенка x2
        for i in 1..(n - count_dublicats) - 1 {
            if reverse {
                indices.push([
                    x2,
                    x2 + i as u32 + 1,
                    x2 + i as u32,
                ]);
            } else {
                indices.push([
                    x2,
                    x2 + i as u32,
                    x2 + i as u32 + 1,
                ]);
            }
        }
    }
    ///
    /// Отзеркаливание по Y и сложение с исходной фигурой
    fn mirror_y(&self, last_figure: &TriMesh) -> Result<TriMesh, Error> {
        let mirrored_vertices: Vec<OPoint<f64, Const<3>>> = last_figure
            .vertices()
            .iter()
            .map(|v| {
                OPoint::<f64, Const<3>>::new(v.x, v.y, v.z)
            })
            .collect();
        let mirrored_indices = last_figure.indices().to_vec();
        match TriMesh::new(mirrored_vertices, mirrored_indices) {
            Ok(mirrored_trimesh) => {
                match self.create_manifold(last_figure.vertices(), last_figure.indices()) {
                    Ok(manifold_original) => {
                        match self.create_manifold(mirrored_trimesh.vertices(), mirrored_trimesh.indices()) {
                            Ok(manifold_mirrored) => {
                                match compute_boolean(&manifold_original, &manifold_mirrored, prelude::OpType::Add) {
                                    Ok(combined_manifold) => {
                                        match self.manifold_to_trimesh(combined_manifold) {
                                            Ok(combined_trimesh) => {
                                                Ok(combined_trimesh)
                                            },
                                            Err(e) => { log::error!("Failed to convert combined manifold to trimesh: {}", e); return Err(e); }
                                        }
                                    },
                                    Err(e) => { log::error!("Failed to combine original and mirrored: {}", e); return Err(e.into()); }
                                }
                            },
                            Err(e) => { log::error!("Failed to create manifold from mirrored: {}", e); return Err(e); }
                        }
                    },
                    Err(e) => { log::error!("Failed to create manifold from original: {}", e); return Err(e); }
                }
            },
            Err(e) => {
                log::error!("Failed to create mirrored TriMesh: {}", e);
                return Err("Failed to create mirrored TriMesh".into());
            }
        }
    }
    ///
    /// Создание [Manifold]
    /// - `vertices` - вершины фигуры
    /// - `indices` - массив индексов треугольников фигуры
    fn create_manifold(&self, vertices: &[OPoint<f64, Const<3>>], indices: &[[u32; 3]]) -> Result<Manifold,Error> {
        let mut all_coords = Vec::new();
        for point in vertices {
            all_coords.push(point.x);
            all_coords.push(point.y);
            all_coords.push(point.z);
        }
        let mut all_indx = Vec::new();
        for triangle_indx in indices {
            for indx in triangle_indx {
                all_indx.push(*indx as usize);
            }
        }
        match Manifold::new(&all_coords, &all_indx) {
            Ok(manifold) => {
                return Ok(manifold)
            },
            Err(e) => return Err(e.into()),
        }
    }
    ///
    /// Преобразование [Manifold] в [TriMesh]
    /// - `manifold` - [Manifold] для преобразования
    fn manifold_to_trimesh(&self, manifold: Manifold) -> Result<TriMesh, Error> {
        let vertices: Vec<OPoint<f64, Const<3>>> =
            manifold.ps.iter()
                .map(|p| OPoint::<f64, Const<3>>::new(p.x, p.y, p.z))
                .collect();
        let indices: Vec<[u32; 3]> =
            manifold.get_indices().iter()
                .map(|t| [t[0] as u32, t[1] as u32, t[2] as u32])
                .collect();
        TriMesh::new(vertices, indices)
            .map_err(|e| e.to_string().into())
    }
    ///
    /// Создание и индексирование вершин
    /// - `tanks_3d` - набор блоков координат отсеков
    fn get_vertices_indeces(&self, tanks_3d: Import3DTanksCtx) -> Vec<TriMesh> {
        let mut result: Vec<TriMesh> = Vec::new();
        let mut last_id = 0.0;
        for comp_corner in tanks_3d.compartment_corner_points {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            let x1 = comp_corner.coordinates_x1.0;
            let x2 = comp_corner.coordinates_x2.0;
            let points_x1 = self.convert_to_points_vec(x1, comp_corner.coordinates_x1.1.clone());
            let points_x2 = self.convert_to_points_vec(x2, comp_corner.coordinates_x2.1.clone());
            if x1 < x2 {
                self.connect_points(&mut vertices, &mut indices, &points_x1, &points_x2, true);
            } else {
                self.connect_points(&mut vertices, &mut indices, &points_x1, &points_x2, false);
            }
            // первая фигура
            if result.len() == 0 {
                match TriMesh::new(vertices, indices) {
                    Ok(trimesh) => {
                        result.push(trimesh);
                    },
                    Err(e) => log::error!("Error to create TriMesh: {}", e),
                }
            } else {
                if last_id == comp_corner.id {
                    match self.create_manifold(&vertices, &indices) {
                        Ok(manifold_curr) => {
                            // последняя фигура
                            let last_figure = result.pop().unwrap();
                            match self.create_manifold(&last_figure.vertices().to_vec(), &last_figure.indices().to_vec()) {
                                Ok(manifold_last) => {
                                    if x1 < x2 { // сложение
                                        match compute_boolean(&manifold_last, &manifold_curr, prelude::OpType::Add) {
                                            Ok(add_res) => {
                                                // преобразование рез-та
                                                match self.manifold_to_trimesh(add_res) {
                                                    Ok(trimesh) => {
                                                        result.push(trimesh);
                                                    },
                                                    Err(e) => log::error!("Enable to convert manifold to trimesh: {}", e),
                                                }
                                            },
                                            Err(e) => log::error!("Error to subtract figures: {}", e),
                                        }
                                    } else { // вычитание
                                        match compute_boolean(&manifold_last, &manifold_curr, prelude::OpType::Subtract) {
                                            Ok(substract_res) => {
                                                // преобразование рез-та
                                                match self.manifold_to_trimesh(substract_res) {
                                                    Ok(trimesh) => {
                                                        result.push(trimesh);
                                                    },
                                                    Err(e) => log::error!("Enable to convert manifold to trimesh: {}", e),
                                                }
                                            },
                                            Err(e) => log::error!("Error to subtract figures: {}", e),
                                        }
                                    }
                                },
                                Err(e) => log::error!("Error to create Manifold: {}", e),
                            }
                        },
                        Err(e) => log::error!("Error to create Manifold: {}", e),
                    }
                } else {
                    if tanks_3d.compartment_id_to_reverse.contains(&(-last_id)) {
                        let last_figure = result.pop().unwrap();
                        match self.mirror_y(&last_figure) {
                            Ok(trimesh) => result.push(trimesh),
                            Err(e) => {
                                log::error!("Error to mirror TriMesh: {}", e);
                                result.push(last_figure);
                            }
                        }
                    }
                    match TriMesh::new(vertices, indices) {
                        Ok(trimesh) => {
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
    fn substract_ship_model(&self, ship_model: TriMesh, tanks: Vec<TriMesh>) -> Option<TriMesh> {
        None
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
                let model_3d = ContextRead::<ConvertModelToTrimeshCtx>::read(&ctx).clone();
                // let result = self.substract_ship_model(
                //     model_3d.surface_outer_body.unwrap(), 
                //     self.get_vertices_indeces(tanks_3d)
                // );
                ctx.write(
                    ConvertTanksToTrimeshCtx {
                        compartment_corner_points: None,
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
