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
use sal_core::{dbg::Dbg, error::Error};
use crate::{
    algorithm::{
        entities::{
            Position, model_cached::{
                DisplacementShape, 
                Shape
            }
        }, 
        eval::room_element_report::room_element_report_ctx::RoomElementReportCtx
    }, 
    kernel::{
        Eval, 
        types::{
            Arc, 
            RwLock, 
            eval_result::EvalResult
        }
    }, 
    prelude::ContextWrite
};
///
/// Алгоритм формирования
/// отчёта по "Элементы помещений"
/// - `tank` - путь к файлу модели отсека
/// - `style` - путь к стилю .css
pub struct RoomElementReportEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
    shape: Arc<RwLock<DisplacementShape>>,
    tank: PathBuf,
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
        tank: PathBuf,
        style: PathBuf,
        shape: Arc<RwLock<DisplacementShape>>,
    ) -> Self {
        let dbg = Dbg::new(parent, "RoomElementReportEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
            shape: shape.clone(),
            tank,
            style
        }
    }
    ///
    /// Расчёт N строки 
    /// второй таблицы
    pub fn room_element_report(
        &self,
        heel: f64,
        trim: f64,
        draught: f64,
    ) -> Result<(f64, Position, f64, Position, f64, f64, f64, f64), Error> {
        match self.shape.read().displacement(heel, trim, draught) {
            Ok((volume, volume_center)) => {
                match self.shape.read().waterline_area(heel, trim, draught) {
                    Ok((square_waterline, center_waterline)) => {
                        match self.shape.read().inertia(heel, trim, draught) {
                            Ok((ix, iy)) => {
                                match self.shape.read().draught_size(heel, trim, draught) {
                                    Ok((_,_,height,mins_z)) => {
                                        Ok(
                                            (
                                                volume, 
                                                volume_center, 
                                                square_waterline, 
                                                center_waterline, 
                                                ix, 
                                                iy, 
                                                height, 
                                                mins_z
                                            )
                                        )
                                    },
                                    Err(e) => {
                                        log::error!("Error: {:?} of calculation model size for draught level: {:?}", e, draught);
                                        Err(e)
                                    }
                                }
                            },
                            Err(e) => {
                                log::error!("Error: {:?} of calculation inertia for draught level: {:?}", e, draught);
                                Err(e)
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Error: {:?} of calculation waterline_area for draught level: {:?}", e, draught);
                        Err(e)
                    },
                }
            },
            Err(e) => {
                log::error!("Error: {:?} of calculation displacement for draught level: {:?}", e, draught);
                Err(e)
            },
        }  
    }
    ///
    /// Создание баз таблиц
    /// и заполнение первой таблицы
    fn create_base_table(
        &self,
        style_path: &PathBuf,
        id_tank: usize, 
        name_tank: String, 
        type_tank: String, 
        location_area: usize,
        permeability_coefficient: f64,
        full_volume: f64,
        draught_level: f64,
    ) -> Document {
        Document::new()
        .title("Отчёт по элементам помещений")
        .style(std::fs::read_to_string(style_path).expect("Error to read `style.css` file"))
        .header(|header| header
            .class("main-header")
            .h1(|el| el
                .class("main-title")
                .text(format!(r#"Отчет "Элементы помещения": {:?} {:?} {:?}"#, type_tank, id_tank, name_tank))
            )
            .h2(|el| el
                .class("subtitle")
                .text(format!("Уровень от ОП: {:.3}", draught_level))
            )
            .text(|el| el 
                .text(" Внимание! Углы поворота помещения задаются в связанной 
                с судном системе координат. При ненулевых значениях углов 
                крена и дифферента судна параметр «Уровень от основной плоскости (ОП)» 
                для данного помещения может отличаться от его значений в условиях прямого 
                положения судна (ровный киль).")
            )
        )
        .section(|section| section  // первая таблица
            .class("section-1")
            .h2(|el| el
                .class("section-1-h2")
            )
            .el(table(), |table| table
                .el(thead(), |thead| thead
                    .el(tr(), |fifth_row| fifth_row
                        .el(th(), |th| th
                            .text("Район расположения, шп.")
                        )
                        .el(th(), |th| th
                            .text(&location_area)
                        )
                    )
                    .el(tr(), |sixth_row| sixth_row
                        .el(th(), |th| th
                            .text("Коэффициент проницаемости")
                        )
                        .el(th(), |th| th
                            .text(&permeability_coefficient)
                        )
                    )
                    .el(tr(), |seventh_row| seventh_row
                        .el(th(), |th| th
                            .text("Объем нетто [м³]")
                        )
                        .el(th(), |th| th
                            .text(&format!("{:.3}", full_volume))
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
        &self,
        doc: Document, 
        level_step: f64,
        heel: f64,
        trim: f64,
        permeability_coefficient: f64,
    ) -> Document {
        let draught_steps = match self.shape
        .read()
        .draught_by_step(level_step, heel, trim) {
            Ok(draught_steps) => draught_steps,
                Err(err) => {
                vec![]
            }
        };
        let result = doc
        .section(|section| {
            section
                .class("section-2")
            .el(table(), |table| {
                let table = table
                .el(thead(), |thead| {
                    thead
                    .el(tr(), |first_row| {
                        first_row
                            .el(th(), |th| {
                                th
                                    .attr(colspan(), "4")
                                    .text("Крен [град]")
                            })
                            .el(th(), |th| { 
                                th
                                    .text(format!("{:.2}", &heel))
                            })
                            .el(th(), |th| {
                                th
                                    .attr(colspan(), "5")
                                    .text("Дифферент [град]")
                            })
                            .el(th(), |th| { 
                                th
                                    .text(format!("{:.2}",&trim))
                            })
                    })
                });
                table                
            })
        })
        .section(|section| {
            section
                .class("section-3")
            .el(table(), |table| {
                let mut table = table
                .el(thead(), |thead| { 
                    thead
                    .el(tr(), |third_row| {
                        third_row
                            .el(th(), |th| {
                                th.attr(colspan(), "2")
                                .text("Уровень")
                            })
                            .el(th(), |th| {
                                th.attr(rowspan(), "3")
                                .text("Объём [м³]")
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
                            .el(th(), |th| th.attr(rowspan(), "2").text("от нижней точки помещения [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("от ОП [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("X [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("Y [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("Z [м]"))
                            .el(th(), |th| th.attr(rowspan(), "2").text("Площадь [м²]"))
                            .el(th(), |th| th.attr(colspan(), "2").text("Координаты центра площади"))
                            .el(th(), |th| th.attr(colspan(), "2").text("Момент инерции"))
                    })
                    .el(tr(), |fifth_row| {
                        fifth_row
                            .el(th(), |th| th.text("X [м]"))
                            .el(th(), |th| th.text("Y [м]"))
                            .el(th(), |th| th.text("IX [м⁴]"))
                            .el(th(), |th| th.text("IY [м⁴]"))
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
                for i in 0..draught_steps.len() {
                    match self.room_element_report(heel, trim, draught_steps[i]) {
                        Ok((
                            volume, 
                            volume_center, 
                            square_waterline, 
                            center_waterline, 
                            ix, 
                            iy, 
                            _, 
                            mins_z
                        )) => {
                            if i == 0 {
                                table = table.el(tr(), |row| row
                                    .el(th(), |th| th
                                        .text(format!("0.00")) // от нижней точки помещения, [м]
                                    )
                                    .el(th(), |th| th
                                        .text(format!("{:.3}", mins_z)) // от ОП, [м]
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
                            }
                            else if i + 1 == draught_steps.len() {
                                table = table.el(tr(), |row| row
                                    .el(th(), |th| th
                                        .text(format!("{:.2}", level_step * (i as f64))) // от нижней точки помещения, [м]
                                    )
                                    .el(th(), |th| th
                                        .text(format!("{:.3}", mins_z + level_step * (i as f64))) // от ОП, [м]
                                    )
                                    .el(th(), |th| th
                                        .text(format!("{:.2}", volume * permeability_coefficient)) // Объём, [м³]
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
                                        .text(format!("-")) // Площадь, [м²]
                                    ) // Координаты центра площади
                                    .el(th(), |th| th
                                        .text(format!("-")) // X, [м]
                                    )
                                    .el(th(), |th| th
                                        .text(format!("-")) // Y, [м]
                                    ) // Момент инерции
                                    .el(th(), |th| th
                                        .text(format!("-")) // IX, [м⁴]
                                    )
                                    .el(th(), |th| th
                                        .text(format!("-")) // IY, [м⁴]
                                    )
                                );
                                break;  
                            }
                            else {
                                table = table.el(tr(), |row| row
                                    .el(th(), |th| th
                                        .text(format!("{:.2}", draught_steps[i] - draught_steps[0])) // от нижней точки помещения, [м]
                                    )
                                    .el(th(), |th| th
                                        .text(format!("{:.3}", draught_steps[i] - draught_steps[0] + mins_z)) // от ОП, [м]
                                    )
                                    .el(th(), |th| th
                                        .text(format!("{:.2}", volume * permeability_coefficient)) // Объём, [м³]
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
                                        .text(format!("{:.2}", square_waterline)) // Площадь, [м²]
                                    ) // Координаты центра площади
                                    .el(th(), |th| th
                                        .text(format!("{:.3}", center_waterline.x())) // X, [м]
                                    )
                                    .el(th(), |th| th
                                        .text(format!("{:.3}", center_waterline.y())) // Y, [м]
                                    ) // Момент инерции
                                    .el(th(), |th| th
                                        .text(format!("{:.2}", ix)) // IX, [м⁴]
                                    )
                                    .el(th(), |th| th
                                        .text(format!("{:.2}", iy)) // IY, [м⁴]
                                    )
                                );
                            }
                        },
                        Err(err) => log::error!("Error to calculate draught eval on draught: {:?} - {:?}", draught_steps[i], err),
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
                let name_tank = String::from("Наименование танка");
                let type_tank = String::from("Тип танка");
                let location_area = 0;
                let permeability_coefficient = 1.0;
                let heel = 0.0;
                let trim = 10.0;
                let level_step = 0.25; // шаг уровня заполняемости
                let result = match self.shape.read().size() {
                    Ok(size) => {
                        match self.shape.read().properties() {
                            Ok(full_volume) => {
                                ctx.write(
                                    RoomElementReportCtx {
                                        result: self.solve_table_2(
                                            self.create_base_table(
                                                &self.style,
                                                id_tank, 
                                                name_tank, 
                                                type_tank, 
                                                location_area, 
                                                permeability_coefficient,
                                                full_volume.0,
                                                size.3
                                            ),
                                            level_step,
                                            heel,
                                            trim,
                                            permeability_coefficient,
                                        ).build(),
                                    }
                                )  
                            },
                            Err(err) => Err(err),
                        }
                      
                    },
                    Err(err) => Err(err),
                };
                result
            }
            Err(err) => Err(err),
        }
    }
}
