use html_builder::{Document, colspan, rowspan, table, th, thead, tr};
use sal_core::dbg::Dbg;
use crate::{algorithm::eval::room_element_report::room_element_report_ctx::RoomElementReportCtx, kernel::{
    Eval, 
    types::eval_result::EvalResult
}, prelude::ContextWrite};
///
/// Алгоритм формирования
/// отчёта по "Элементы помещений"
pub struct RoomElementReportEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl RoomElementReportEval {
    ///
    /// Новый экземпляр [RoomElementReportEval]
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "RoomElementReportEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
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
                            .text("Крен, {градус")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "1")
                            .text(&tilt)
                        )
                    )
                    .el(tr(), |second_row| second_row
                        .el(th(), |th| th
                            .attr(colspan(), "2")
                            .text("Дифферент, градус")
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
                            .text("Объём, м³")
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
                            .text("от нижней точки помещения, м")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("от основной плоскости, м")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("X, м")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("Y, м")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("Z, м")
                        )
                        .el(th(), |th| th
                            .attr(rowspan(), "2")
                            .text("Площадь, м²")
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
                            .text("X, м")
                        )
                        .el(th(), |th| th
                            .text("Y, м")
                        )
                        .el(th(), |th| th
                            .text("IX, м⁴")
                        )
                        .el(th(), |th| th
                            .text("IY, м⁴")
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
    fn solve_table_2(doc: Document, level: f64, level_step: f64) -> Document {
        let mut result = doc;
        let mut curr_level = 0.0;
        while curr_level < level {
            result = result.el(tr(), |row| row
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
                .el(th(), |th| th
                    .text("")
                )
            );
            curr_level += level_step
        }
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
                let permeability_coefficient = 0.0;
                let tilt = 0.0;
                let trim = 0.0;
                let level = 0.0; // уровень заполняемости 
                let level_step = 0.0; // шаг уровня заполняемости 
                ctx.write(
                    RoomElementReportCtx {
                        result: Self::solve_table_2(
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
