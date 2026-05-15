use std::{path::PathBuf, sync::Arc};
use html_builder::{
    Document,
    colspan,
    table,
    th,
    tr
};
use sal_3dlib::{HullSlicer, load_stl};
use sal_core::{dbg::Dbg, error::Error};
use crate::{
    algorithm::{
        eval::reports::bonjan_report::bonjan_report_ctx::BonjanReportCtx
    }, 
    kernel::{
        Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::ContextWrite
};
///
/// Класс по генерации HTML-отчёт «Масштаб Бонжана» на основе 3D-модели корпуса судна.
/// - `dbg` — объект отладки для формирования диагностических сообщений.
/// - `ctx` — следующий вычислительный контекст, в который записывается результат.
/// - `ship_model_path` — путь к STL-файлу с трёхмерной моделью корпуса судна.
/// - `style` — путь к CSS-файлу, используемому для оформления HTML-отчёта.
/// - `draught_step` — шаг изменения осадки судна в метрах.
/// - `samples` — список координат вдоль оси X, определяющих положения шпаций.
pub struct BonjanReportEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
    ship_model_path: PathBuf,
    style: PathBuf,
    draught_step: f64,
    samples: Vec<f64>,
}
//
//
impl BonjanReportEval {
    ///
    /// Создание нового экземпляра класса [BonjanReportEval]
    /// - `parent` — имя родительского компонента для системы отладки.
    /// - `ctx` — вычислительный контекст, в который будет записан HTML-отчёт.
    /// - `ship_model_path` — путь к STL-модели корпуса судна.
    /// - `style` — путь к CSS-файлу со стилями отчёта.
    /// - `draught_step` — шаг изменения осадки в метрах.
    /// - `samples` — координаты шпаций вдоль продольной оси судна.
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
        ship_model_path: PathBuf,
        style: PathBuf,
        draught_step: f64,
        samples: Vec<f64>,
    ) -> Self {
        Self { 
            ctx: Box::new(ctx), 
            style,
            ship_model_path,
            dbg: Dbg::new(parent, "BonjanReportEval"),
            draught_step,
            samples, 
        }
    }
    ///
    /// Инициализация базового HTML-документ отчёта.
    /// Создание документа, задание заголовка страницы, подключение CSS-стиля
    /// и формирование основного заголовка отчёта.
    fn init_document(
        &self,
    ) -> Document {
        Document::new()
        .title("Отчёт по масштабу бонжана")
        .style(std::fs::read_to_string(self.style.clone()).expect("Error to read `style.css` file"))
        .header(|header| header
            .class("main-header")
            .h1(|el| el
                .class("main-title")
                .text(format!(r#"Отчет "Масштабу бонжана"#))
            )
        )
    }
    ///
    /// Калькуляция данных для таблицы масштаба Бонжана.
    /// Сам алгоритм берется из библиотеки [sal_3dlib]
    /// Результатом является кортеж:
    /// - `Vec<f64>` — координаты шпаций;
    /// - `Vec<f64>` — значения осадки;
    /// - `Vec<Vec<(f64, f64)>>` — результаты расчётов для каждой шпации,
    ///   где каждый элемент содержит пару `(осадка, значение)`.
    fn calculate_table_rows(
        &self
    ) -> (Vec<f64>, Vec<f64>, Vec<Vec<(f64,f64)>>) {
        let ship_mesh = load_stl(&self.ship_model_path, 1000.).expect("Error to load ship model");
        let aabb = ship_mesh.local_aabb();
        let mut samples = self.samples.clone();
        if aabb.mins.x < self.samples[0] {
            samples.insert(0,aabb.mins.x);
        } 
        if aabb.maxs.x > self.samples[self.samples.len() - 1] {
            samples.push(aabb.maxs.x);
        }
        let mut draught_steps = Vec::new();
        let mut curr_draught = 0.0;
        loop {
            if curr_draught > 15.0 {
                draught_steps.push(15.0);
                break;
            }
            draught_steps.push(curr_draught);
            curr_draught += self.draught_step
        }
        let ship_slices = HullSlicer::new(Arc::new(ship_mesh)).slice(&samples);
        let mut result_volumes = Vec::new();
        for ship_slice in ship_slices {
            result_volumes.push(ship_slice.calculate_displacements(&draught_steps));
        }
        (samples, draught_steps, result_volumes)
    }
    ///
    /// Формирование основной таблицы отчёта
    /// Таблица содержит:
    /// - строку заголовков с диапазонами шпаций;
    /// - столбец значений осадки;
    /// - рассчитанные значения масштаба Бонжана для каждой шпации.
    /// 
    /// - `doc` — исходный HTML-документ.
    fn init_table(
        &self,
        doc: Document,
    ) -> Result<Document, Error> {
        let (samples, draught_steps, result_volumes) = self.calculate_table_rows();
        Ok(doc.section(|section| section
        .class("section-1")
        .el(table(), |table| {
            let mut table = table
            .class("report-table")
            .el(tr(), |first_row| first_row
                .el(th(), |th| th
                    .attr(colspan(), "1")
                    .text(
                        "Осадка"
                    )
                )
                .el(th(), |th| th
                        .attr(colspan(), "22")
                        .text("Шпации")
                )
            )
            .el(tr(), |second_row| {
                let mut second_row = second_row;
                second_row = second_row.el(th(), |th| th.text("[м]"));
                for i in 1..samples.len() {
                    second_row = second_row.el(th(), |th| th.text(format!("{:.2}-{:.2}", samples[i-1], samples[i])));
                }
                second_row
            });
            for (i, draught_step) in draught_steps.iter().enumerate() {
                table = table.el(tr(), |result_row| {
                    let mut result_row = result_row;
                    result_row = result_row.el(th(), |td| td.text(format!("{:.2}", draught_step)));
                    for result_volume in result_volumes.iter() {
                        if let Some((_, volume)) = result_volume.get(i) {
                            result_row = result_row.el(th(), |td| {
                                td.text(format!("{:.2}", volume))
                            });
                        }
                    }
                    result_row
                });
            }
            table
        })))
    }
}
//
//
impl Eval<(), EvalResult> for BonjanReportEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                match self.init_table(
                    self.init_document()
                ) {
                    Ok(report) => {
                        ctx.write(
                            BonjanReportCtx {
                                result: report.build(),
                            }
                        ) 
                    },
                    Err(err) => Err(err),
                }
 
            }
            Err(err) => Err(err),
        }
    }
}