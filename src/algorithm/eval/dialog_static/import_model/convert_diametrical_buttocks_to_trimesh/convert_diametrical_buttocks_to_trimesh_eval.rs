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
use crate::algorithm::eval::import_model::convert_diametrical_buttocks_to_trimesh::convert_diametrical_buttocks_to_trimesh_ctx::ConvertDiametricalButtocksToTrimeshCtx;
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
    /// Преобразование координат в набор точек [Point]
    /// - `vec_x_y_z` - набор (X,Y,Z) координат
    fn convert_to_points_vec(
        &self,
        vec_x_y_z: &Vec<(f64, f64, f64)>, 
    ) -> Vec<Point<f64>>{
        let mut frame: Vec<Point<f64>> = Vec::new();
        for (x, y, z) in vec_x_y_z {
            let point = Point::new(*x, *z, *y);
            frame.push(point);
        }
        return frame;
    }
    ///
    /// Закрытие сэмпла веером
    /// - `reverse` - ориентация нормалей
    /// - `x2` - индекс начала вершин закрываемого сэмпла
    /// - `vertices` - вершины 3D модели
    /// - `indices` - массив индексов треугольников фигуры
    /// - `points` - вершины закрываемого сэмпла
    /// - `n` - кол-во точек закрываемого сэмпла
    fn build_wall(
        &self, 
        reverse: bool, 
        x2: u32, 
        indices: &mut Vec<[u32; 3]>, 
        vertices: &mut Vec<OPoint<f64, Const<3>>>,
        points: &Vec<OPoint<f64, Const<3>>>, 
        n: usize,
    ) {
        let centroid = self.calculate_centroid(points);
        let center_idx = vertices.len() as u32;
        vertices.push(centroid);
        for i in 0..n {
            let next = (i + 1) % n;
            if reverse {
                indices.push([
                    center_idx,
                    x2 + next as u32,
                    x2 + i as u32,
                ]);
            } else {
                indices.push([
                    center_idx,
                    x2 + i as u32,
                    x2 + next as u32,
                ]);
            }
        }
    }   
    ///
    /// Вычисляется центроида сэмпла
    /// - `points` - точки у которых надо найти центроид
    fn calculate_centroid(
        &self, 
        points: &[Point<f64>]
    ) -> Point<f64> {
        let sum = points.iter().fold(Point::new(0.0, 0.0, 0.0), |acc, p| {
            Point::new(acc.x + p.x, acc.y + p.y, acc.z + p.z)
        });
        let count = points.len() as f64;
        Point::new(sum.x / count, sum.y / count, sum.z / count)
    }
    ///
    /// Создание и индексирование вершин
    /// - `tanks_3d` - набор блоков координат отсеков
    /// - `target_points` - кол-во точек на шпангоут для интерполяции
    fn convert_diametrical_buttocks(&self, tanks_3d: ImportModelInitialPointsCtx) -> (Option<TriMesh>, Option<TriMesh>) {
        // stern buttocks
        let stern_points = self.convert_to_points_vec(&tanks_3d.stern_block);
        let mut stern_vertices = Vec::new();
        stern_vertices.extend_from_slice(&stern_points);
        let mut stern_indices = Vec::new();
        self.build_wall(false, 0, &mut stern_indices, &mut stern_vertices, &stern_points, stern_points.len());
        // nasal buttocks
        let nasal_points = self.convert_to_points_vec(&tanks_3d.nasal_block);
        let mut nasal_vertices = Vec::new();
        nasal_vertices.extend_from_slice(&nasal_points);
        let mut nasal_indices = Vec::new();
        self.build_wall(false, 0, &mut nasal_indices, &mut nasal_vertices, &nasal_points, nasal_points.len());
        let mut stern_buttocks = None;
        let mut nasal_buttocks = None;
        match TriMesh::new(stern_vertices, stern_indices) {
            Ok(res) => stern_buttocks = Some(res),
            Err(e) => log::error!("Error to create stern buttocks: {:?}", e),
        }
        match TriMesh::new(nasal_vertices, nasal_indices) {
            Ok(res) => nasal_buttocks = Some(res),
            Err(e) => log::error!("Error to create nasal buttocks: {:?}", e),
        }
        (stern_buttocks, nasal_buttocks)
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
                let (stern_buttocks, nasal_buttocks) = self
                    .convert_diametrical_buttocks(model_3d);
                ctx.write(
                    ConvertDiametricalButtocksToTrimeshCtx {
                        stern_buttocks,
                        nasal_buttocks,
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