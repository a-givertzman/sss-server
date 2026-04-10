use std::{io::Write, path::PathBuf};

use html_builder::{Document, colspan, rowspan, table, th, thead, tr};
use nalgebra::{UnitQuaternion, UnitVector3, Vector3};
use parry3d_f64::{bounding_volume::Aabb, math::{Isometry, Vector}, shape::{Shape, TriMesh}};
use sal_core::{dbg::Dbg, error::Error};
use crate::{algorithm::{entities::model_cached::{rotate, square, volume}, eval::room_element_report::room_element_report_ctx::RoomElementReportCtx}, kernel::{
    Eval, 
    types::eval_result::EvalResult
}, prelude::ContextWrite};
///
/// Алгоритм формирования
/// отчёта по "Элементы помещений"
pub struct RoomElementReportEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
    tank: TriMesh,
}
//
//
impl RoomElementReportEval {
    ///
    /// Новый экземпляр [RoomElementReportEval]
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
        tank: TriMesh
    ) -> Self {
        let dbg = Dbg::new(parent, "RoomElementReportEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
            tank
        }
    }
    fn create_base_table(
        id_tank: usize, 
        name_tank: String, 
        type_tank: String, 
        location_area: usize,
        permeability_coefficient: f64,
        tilt: f64,
        trim: f64
    ) -> Document {
        Document::new()
        .title("Отчёт по элементам помещений")
        .header(|header| header
            .class("main-header")
            .h1(|el| el
                .class("main-title")
                .text("Отчёт по элементам помещений")
            )
        )
        .section(|section| section  // первая таблица
            .class("section-1")
            .h2(|el| el
                .class("section-1-h2")
                .text("Таблица 1")
            )
            .el(table(), |table| table
                .el(thead(), |thead| thead
                    .el(tr(), |first_row| first_row
                        .el(th(), |th| th
                            .text("Код помещения")
                        )
                        .el(th(), |th| th
                            .text(&id_tank)
                        )
                    )
                    .el(tr(), |first_row| first_row
                        .el(th(), |th| th
                            .text("Наименование помещения")
                        )
                        .el(th(), |th| th
                            .text(name_tank)
                        )
                    )
                    .el(tr(), |first_row| first_row
                        .el(th(), |th| th
                            .text("Тип помещения")
                        )
                        .el(th(), |th| th
                            .text(&type_tank)
                        )
                    )
                    .el(tr(), |first_row| first_row
                        .el(th(), |th| th
                            .text("Район расположения, шп.")
                        )
                        .el(th(), |th| th
                            .text(&location_area)
                        )
                    )
                    .el(tr(), |first_row| first_row
                        .el(th(), |th| th
                            .text("Коэффициент проницаемости")
                        )
                        .el(th(), |th| th
                            .text(&permeability_coefficient)
                        )
                    )
                )
            )
        ) 
        .section(|section| section // вторая таблица
            .class("section-2")
            .h2(|el| el
                .class("section-2-h2")
                .text("Таблица 2")
            )
            .el(table(), |table| table
                .el(thead(), |thead| thead
                    .el(tr(), |first_row| first_row
                        .el(th(), |th| th
                            .attr(colspan(), "2")
                            .text("Крен, [град]")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "1")
                            .text(&tilt)
                        )
                    )
                    .el(tr(), |second_row| second_row
                        .el(th(), |th| th
                            .attr(colspan(), "2")
                            .text("Дифферент, [град]")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "1")
                            .text(&trim)
                        )
                    )
                    .el(tr(), |third_row| third_row
                        .el(th(), |th| th
                            .attr(colspan(), "2")
                            .text("Уровень")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "3")
                            .text("Объём, [м³]")
                        )
                        .el(th(), |th| th
                            .attr(colspan(), "3")
                            .text("Координаты центра объёма")
                        )
                        .el(th(), |th| th
                            .attr(colspan(), "5")
                            .text("Элементы свободной поверхности")
                        )
                    )
                    .el(tr(), |fourth_row| fourth_row
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("от нижней точки помещения, [м]")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("от ОП, [м]")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("X, [м]")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("Y, [м]")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("Z, [м]")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("Площадь, [м²]")
                        )
                        .el(th(), |th| th
                            .attr(colspan(), "2")
                            .text("Координаты центра площади")
                        )
                        .el(th(), |th| th
                            .attr(colspan(), "2")
                            .text("Момент инерции")
                        )
                    )
                    .el(tr(), |fifth_row| fifth_row
                        .el(th(), |th| th
                            .text("X, [м]")
                        )
                        .el(th(), |th| th
                            .text("Y, [м]")
                        )
                        .el(th(), |th| th
                            .text("IX, [м⁴]")
                        )
                        .el(th(), |th| th
                            .text("IY, [м⁴]")
                        )
                    )
                    .el(tr(), |sixth_row| sixth_row
                        .el(th(), |th| th
                            .text("1")
                        )
                        .el(th(), |th| th
                            .text("2")
                        )
                        .el(th(), |th| th
                            .text("3")
                        )
                        .el(th(), |th| th
                            .text("4")
                        )
                        .el(th(), |th| th
                            .text("5")
                        )
                        .el(th(), |th| th
                            .text("6")
                        )
                        .el(th(), |th| th
                            .text("7")
                        )
                        .el(th(), |th| th
                            .text("8")
                        )
                        .el(th(), |th| th
                            .text("9")
                        )
                        .el(th(), |th| th
                            .text("10")
                        )
                        .el(th(), |th| th
                            .text("11")
                        )
                    )
                )
            )
        )
    }
    ///
    /// Добавление результатов расчётов в таблицу 2
    /// по шагу уровня заполняемости
    fn solve_table_2(
        init_tank_aabb: Aabb,
        tank: TriMesh,
        doc: Document, 
        level: f64, 
        level_step: f64
    ) -> Document {
        let figure_aabb = tank.compute_local_aabb();
        let mut result = doc;
        let mut curr_level = 0.0;
        match write_stl(&PathBuf::from("src\\tests\\unit\\algorithm\\reports\\tank.stl"), &tank) {
            Ok(_) =>  {},
            Err(_) => {},
        }
        while curr_level < level {
            let z_to_cut = curr_level + figure_aabb.mins.z;
            match tank.split(&Isometry::identity(), &Vector::z_axis(), z_to_cut, 1e-6) {
                parry3d_f64::query::SplitResult::Pair(a, _) => {
                    match write_stl(&PathBuf::from("src\\tests\\unit\\algorithm\\reports\\splited_mesh.stl"), &a) {
                        Ok(_) =>  {},
                        Err(_) => {},
                    }
                },
                parry3d_f64::query::SplitResult::Negative => {

                },
                parry3d_f64::query::SplitResult::Positive => {

                },
            }
            result = result.el(tr(), |row| row
                .el(th(), |th| th
                    .text(figure_aabb.maxs.y - figure_aabb.mins.y) // от нижней точки помещения, [м]
                )
                .el(th(), |th| th
                    .text(figure_aabb.maxs.y - figure_aabb.mins.y + (figure_aabb.mins.y  - init_tank_aabb.mins.y).abs()) // от ОП, [м]
                )
                .el(th(), |th| th
                    .text(volume(&tank)) // Объём, [м³]
                ) // Координаты центра объёма
                .el(th(), |th| th
                    .text(figure_aabb.center().x) // X, [м]
                )
                .el(th(), |th| th
                    .text(figure_aabb.center().y) // Y, [м]
                )
                .el(th(), |th| th
                    .text(figure_aabb.center().z) // Z, [м]
                )
                .el(th(), |th| th
                    .text(square(&tank)) // Площадь, [м²]
                ) // Координаты центра площади
                .el(th(), |th| th
                    .text("") // X, [м]
                )
                .el(th(), |th| th
                    .text("") // Y, [м]
                ) // Момент инерции
                .el(th(), |th| th
                    .text("") // IX, [м⁴]
                )
                .el(th(), |th| th
                    .text("") // IY, [м⁴]
                )
            );
            curr_level += level_step
        }
        result
    }
}
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
//
//
impl Eval<(), EvalResult> for RoomElementReportEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let id_tank = 0;
                let name_tank = String::new();
                let type_tank = String::new();
                let location_area = 0;
                let permeability_coefficient = 0.0;
                let tilt = 90.0 * 3.14 / 180.0;
                let trim = 0.0 * 3.14 / 180.0;
                let level = 5.0; // уровень заполняемости 
                let level_step = 1.0; // шаг уровня заполняемости
                let init_tank_aabb = self.tank.compute_local_aabb();
                let tank_rotated = rotate(&self.tank, 0.0, tilt, trim);
                ctx.write(
                    RoomElementReportCtx {
                        result: Self::solve_table_2(
                            init_tank_aabb,
                            tank_rotated,
                            Self::create_base_table(
                                id_tank, 
                                name_tank, 
                                type_tank, 
                                location_area, 
                                permeability_coefficient, 
                                tilt, 
                                trim
                            ),
                            level,
                            level_step
                        ).build(),
                    }
                )
            }
            Err(err) => Err(err),
        }
    }
}
