use std::path::PathBuf;
use html_builder::{
    Document,
    colspan,
    rowspan,
    table,
    th,
    thead,
    tr
};
use nalgebra::{UnitQuaternion, UnitVector3, Vector3};
use parry3d_f64::{
    shape::{
        Shape, 
        TriMesh
    }
};
use sal_core::{dbg::Dbg};
use crate::{
    algorithm::{
        entities::model_cached::{DisplacementShape, transform_point}, 
        eval::room_element_report::room_element_report_ctx::RoomElementReportCtx
    }, 
    kernel::{
        Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::ContextWrite
};
///
/// Алгоритм формирования
/// отчёта по "Элементы помещений"
pub struct RoomElementReportEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
    tank: TriMesh,
    style: PathBuf,
}
//
//
impl RoomElementReportEval {
    ///
    /// Новый экземпляр [RoomElementReportEval]
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
        tank: TriMesh,
        style: PathBuf,
    ) -> Self {
        let dbg = Dbg::new(parent, "RoomElementReportEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
            tank,
            style
        }
    }
    ///
    /// Создание баз таблиц
    /// и заполнение первой таблицы
    fn create_base_table(
        style_path: &PathBuf,
        id_tank: usize, 
        name_tank: String, 
        type_tank: String, 
        location_area: usize,
        permeability_coefficient: f64,
    ) -> Document {
        Document::new()
        .title("Отчёт по элементам помещений")
        .style(std::fs::read_to_string(style_path).expect("Error to read `style.css` file"))
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
    }
    ///
    /// Добавление результатов расчётов в таблицу 2
    /// по шагу уровня заполняемости
    fn solve_table_2(
        tank: &TriMesh,
        doc: Document, 
        level_step: f64,
        heel: f64,
        trim: f64,
        permeability_coefficient: f64,
        displacement_shape: DisplacementShape
    ) -> Document {
        let aabb = tank.compute_local_aabb();
        let z_min = aabb.mins;
        let z_max = aabb.maxs;
        let mut curr_level = transform_point(&z_min, heel, trim).z + level_step;
        let max_level = transform_point(&z_max, heel, trim).z;
        let result = doc.section(|section| {
            section
                .class("section-2")
                .h2(|el| {
                    el.class("section-2-h2")
            })
            .el(table(), |table| {
                let mut table = table
                .el(thead(), |thead| { 
                    thead
                    .el(tr(), |first_row| {
                        first_row
                            .el(th(), |th| {
                                th.attr(colspan(), "4")
                                .text("Крен, [град]")
                            })
                            .el(th(), |th| { th
                                .text(&heel)
                            })
                            .el(th(), |th| {
                                th.attr(colspan(), "5")
                                .text("Дифферент, [град]")
                            })
                            .el(th(), |th| { th
                                .text(&trim)
                            })
                    })
                    .el(tr(), |third_row| {
                        third_row
                            .el(th(), |th| {
                                th.attr(colspan(), "2")
                                .text("Уровень")
                            })
                            .el(th(), |th| {
                                th.attr(rowspan(), "3")
                                .text("Объём, [м³]")
                            })
                            .el(th(), |th| {
                                th.attr(colspan(), "3")
                                .text("Координаты центра объёма")
                            })
                            .el(th(), |th| {
                                th.attr(colspan(), "5")
                                .text("Элементы свободной поверхности")
                            })
                    })
                    .el(tr(), |fourth_row| {
                        fourth_row
                            .el(th(), |th| th.attr(rowspan(), "2").text("от нижней точки помещения, [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("от ОП, [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("X, [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("Y, [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("Z, [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("Площадь, [м²]"))
                            .el(th(), |th| th.attr(colspan(), "2").text("Координаты центра площади"))
                            .el(th(), |th| th.attr(colspan(), "2").text("Момент инерции"))
                    })
                    .el(tr(), |fifth_row| {
                        fifth_row
                            .el(th(), |th| th.text("X, [м]"))
                            .el(th(), |th| th.text("Y, [м]"))
                            .el(th(), |th| th.text("IX, [м⁴]"))
                            .el(th(), |th| th.text("IY, [м⁴]"))
                    })
                    .el(tr(), |sixth_row| {
                        sixth_row
                            .el(th(), |th| th.text("1"))
                            .el(th(), |th| th.text("2"))
                            .el(th(), |th| th.text("3"))
                            .el(th(), |th| th.text("4"))
                            .el(th(), |th| th.text("5"))
                            .el(th(), |th| th.text("6"))
                            .el(th(), |th| th.text("7"))
                            .el(th(), |th| th.text("8"))
                            .el(th(), |th| th.text("9"))
                            .el(th(), |th| th.text("10"))
                            .el(th(), |th| th.text("11"))
                    })
                });
                table = table.el(tr(), |row| row
                    .el(th(), |th| th
                        .text(format!("0.0")) // от нижней точки помещения, [м]
                    )
                    .el(th(), |th| th
                        .text(format!("{:.3}", curr_level - level_step)) // от ОП, [м]
                    )
                    .el(th(), |th| th
                        .text(format!("-")) // Объём, [м³]
                    ) // Координаты центра объёма
                    .el(th(), |th| th
                        .text(format!("-")) // X, [м]
                    )
                    .el(th(), |th| th
                        .text(format!("-")) // Y, [м]
                    )
                    .el(th(), |th| th
                        .text(format!("-")) // Z, [м]
                    )
                    .el(th(), |th| th
                        .text(format!("-")) // Площадь, [м²]
                    ) // Координаты центра площади
                    .el(th(), |th| th
                        .text(format!("-")) // X, [м]
                    )
                    .el(th(), |th| th
                        .text(format!("-", )) // Y, [м]
                    ) // Момент инерции
                    .el(th(), |th| th
                        .text(format!("-", )) // IX, [м⁴]
                    )
                    .el(th(), |th| th
                        .text(format!("-", )) // IY, [м⁴]
                    )
                );
                while curr_level <= max_level {
                    match displacement_shape.displacement(heel, trim, curr_level) {
                        Ok((volume, volume_center)) => {
                            match displacement_shape.waterline_area(heel, trim, curr_level) {
                                Ok((square_waterline, center_waterline)) => {
                                    match displacement_shape.inertia(heel, trim, curr_level) {
                                        Ok((ix, iy)) => {
                                            match displacement_shape.draught_size(heel, trim, curr_level) {
                                                Ok((_,_,height,mins_z)) => {
                                                    table = table.el(tr(), |row| row
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", height)) // от нижней точки помещения, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", height + mins_z)) // от ОП, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", volume * permeability_coefficient)) // Объём, [м³]
                                                        ) // Координаты центра объёма
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", volume_center.x())) // X, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", volume_center.y())) // Y, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", volume_center.z())) // Z, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", square_waterline)) // Площадь, [м²]
                                                        ) // Координаты центра площади
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", center_waterline.x())) // X, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", center_waterline.y())) // Y, [м]
                                                        ) // Момент инерции
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", ix)) // IX, [м⁴]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", iy)) // IY, [м⁴]
                                                        )
                                                    );
                                                },
                                                Err(e) => log::error!("Error: {:?} of calculation model size for draught level: {:?}", e, curr_level),
                                            }
                                        },
                                        Err(e) => log::error!("Error: {:?} of calculation inertia for draught level: {:?}", e, curr_level),
                                    }
                                },
                                Err(e) => log::error!("Error: {:?} of calculation waterline_area for draught level: {:?}", e, curr_level),
                            }
                        },
                        Err(e) => log::error!("Error: {:?} of calculation displacement for draught level: {:?}", e, curr_level),
                    }
                    curr_level += level_step
                }
                if curr_level > max_level {
                    match displacement_shape.displacement(-heel, trim, max_level) {
                        Ok((volume, volume_center)) => {
                            match displacement_shape.waterline_area(-heel, trim, max_level) {
                                Ok((square_waterline, center_waterline)) => {
                                    match displacement_shape.inertia(-heel, trim, max_level) {
                                        Ok((ix, iy)) => {
                                            match displacement_shape.draught_size(-heel, trim, max_level) {
                                                Ok((_,_,height,mins_z)) => {
                                                    table = table.el(tr(), |row| row
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", height)) // от нижней точки помещения, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", height + mins_z)) // от ОП, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", volume * permeability_coefficient)) // Объём, [м³]
                                                        ) // Координаты центра объёма
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", volume_center.x())) // X, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", volume_center.y())) // Y, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", volume_center.z())) // Z, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", square_waterline)) // Площадь, [м²]
                                                        ) // Координаты центра площади
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", center_waterline.x())) // X, [м]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", center_waterline.y())) // Y, [м]
                                                        ) // Момент инерции
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", ix)) // IX, [м⁴]
                                                        )
                                                        .el(th(), |th| th
                                                            .text(format!("{:.3}", iy)) // IY, [м⁴]
                                                        )
                                                    );
                                                },
                                                Err(e) => log::error!("Error: {:?} of calculation model size for draught level: {:?}", e, curr_level),
                                            }
                                        },
                                        Err(e) => log::error!("Error: {:?} of calculation inertia for draught level: {:?}", e, curr_level),
                                    }
                                },
                                Err(e) => log::error!("Error: {:?} of calculation waterline_area for draught level: {:?}", e, curr_level),
                            }
                        },
                        Err(e) => log::error!("Error: {:?} of calculation displacement for draught level: {:?}", e, curr_level),
                    }                   
                }                     
                table
            })
        });
        result
    }
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
                let permeability_coefficient = 1.0;
                let heel = 0.0;
                let trim = 10.0;
                let level_step = 0.25; // шаг уровня заполняемости
                ctx.write(
                    RoomElementReportCtx {
                        result: Self::solve_table_2(
                            &self.tank,
                            Self::create_base_table(
                                &self.style,
                                id_tank, 
                                name_tank, 
                                type_tank, 
                                location_area, 
                                permeability_coefficient, 
                            ),
                            level_step,
                            heel,
                            trim,
                            permeability_coefficient,
                            DisplacementShape::new(
                                &self.dbg, 
                                Some(self.tank.clone()), 
                                None, 
                                Some(self.tank.compute_local_aabb().mins.z), 
                                1000.0, 
                                0.0001, 
                                10000,
                            )
                        ).build(),
                    }
                )
            }
            Err(err) => Err(err),
        }
    }
}
