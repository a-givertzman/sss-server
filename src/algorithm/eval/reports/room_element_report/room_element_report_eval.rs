use std::path::PathBuf;
use html_builder::{
    Document, Translation, colspan, rowspan, table, th, thead, tr
};
use std::borrow::Cow;
use nalgebra::{Const, OPoint};
use sal_core::dbg::Dbg;
use sal_sync::sync::RwLock;
use crate::{
    algorithm::{
        entities::model_cached::DisplacementShape, 
        eval::room_element_report::room_element_report_ctx::RoomElementReportCtx
    }, 
    kernel::{
        Eval, 
        types::{Arc, eval_result::EvalResult}
    }, 
    prelude::ContextWrite
};
///
/// Генератор HTML-отчёта по элементам помещения.
///
/// Формирует отчёт по заполнению помещения (танка) для различных
/// значений крена и дифферента.
///
/// Отчёт содержит:
/// - общую информацию о помещении;
/// - таблицу параметров заполнения;
/// - координаты центров объёма;
/// - параметры свободной поверхности;
/// - моменты инерции свободной поверхности.
///
/// Для генерации используется [`html_builder::Document`].
pub struct RoomElementReportEval {
    /// Отладочный контекст.
    dbg: Dbg,
    /// Контекст записи результата вычислений.
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
    /// Геометрическая модель помещения.
    ///
    /// Используется для расчёта:
    /// - объёма;
    /// - центра объёма;
    /// - параметров свободной поверхности.
    shape: Arc<RwLock<DisplacementShape>>,
    /// Путь к CSS-файлу стилей отчёта.
    style: PathBuf,
    /// Список углов крена [град].
    heel: Vec<f64>,
    /// Список углов дифферента [град].
    trim: Vec<f64>,
}
//
//
impl RoomElementReportEval {
    ///
    /// Создание нового экземпляра [`RoomElementReportEval`].
    ///
    /// # Параметры
    ///
    /// - `parent` — имя родительского объекта для системы логирования;
    /// - `ctx` — контекст сохранения результата;
    /// - `style` — путь к CSS-файлу;
    /// - `shape` — геометрическая модель помещения;
    /// - `heel` — список углов крена [град];
    /// - `trim` — список углов дифферента [град].
    ///
    /// # Возвращает
    ///
    /// Новый экземпляр [`RoomElementReportEval`].
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
        style: PathBuf,
        shape: Arc<RwLock<DisplacementShape>>,
        heel: Vec<f64>,
        trim: Vec<f64>
    ) -> Self {
        let dbg = Dbg::new(parent, "RoomElementReportEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
            shape: shape.clone(),
            style,
            heel,
            trim,
        }
    }
    ///
    /// Инициализация HTML-документа отчёта.
    ///
    /// Создаёт:
    /// - заголовок документа;
    /// - подключение CSS-стилей;
    /// - основной заголовок отчёта;
    /// - текстовое описание системы координат.
    ///
    /// # Параметры
    ///
    /// - `style_path` — путь к CSS-файлу;
    /// - `id_tank` — идентификатор помещения;
    /// - `name_tank` — имя помещения;
    /// - `type_tank` — тип помещения.
    ///
    /// # Возвращает
    ///
    /// Инициализированный [`Document`].
    fn init_document(
        &self,
        style_path: &PathBuf,
        id_tank: usize, 
        name_tank: String, 
        type_tank: String,
    ) -> Document {
        let translation = vec![
            (
                "Отчёт по элементам помещений",
                "Room elements report",
            ),
            (
                r#"Отчет "Элементы помещения""#,
                r#"Report "Room elements""#,
            ),
            (
                " Углы и координаты приведены в связанной с судном системе координат. \
                Уровни отсчитываются по оси, проходящей через центр полного объема цистерны.",
                " Angles and coordinates are given in the ship-bound coordinate system. \
                Levels are measured along the axis passing through the centre of the tank's full volume.",
            ),
            ("Район расположения, шп.", "Location area, fr."),
            ("Коэффициент проницаемости", "Permeability coefficient"),
            ("Полный объем нетто [м³]", "Net full volume [m³]"),
            ("Максимальный момент инерции IX [м⁴]", "Maximum moment of inertia IX [m⁴]"),
            ("Крен [град]", "Heel [deg]"),
            ("Дифферент [град]", "Trim [deg]"),
            ("Уровень", "Level"),
            ("Объём [м³]", "Volume [m³]"),
            ("Координаты центра объёма", "Volume centre coordinates"),
            ("Элементы свободной поверхности", "Free surface elements"),
            ("от нижней точки помещения [м]", "from the lowest point of the room [m]"),
            ("от ОП [м]", "from BP [m]"),
            ("X [м]", "X [m]"),
            ("Y [м]", "Y [m]"),
            ("Z [м]", "Z [m]"),
            ("Площадь [м²]", "Area [m²]"),
            ("Координаты центра площади", "Area centre coordinates"),
            ("Момент инерции", "Moment of inertia"),
            ("IX [м⁴]", "IX [m⁴]"),
            ("IY [м⁴]", "IY [m⁴]"),
        ]
        .into_iter()
        .map(|(key, val)| (key.to_string(), val.to_string()));
        Document::new()
        .localize(Translation::new(translation))
        .title("Отчёт по элементам помещений")
        .style(std::fs::read_to_string(style_path).expect("Error to read `style.css` file"))
        .header(|header| header
            .class("main-header")
            .h1(|el| el
                .class("main-title")
                .text(r#"Отчет "Элементы помещения""#)
                .text(format!(r#"": {:?} {:?} {:?}"#, type_tank, id_tank, name_tank))
            )
            .text(|el| el 
                .text(
                    " Углы и координаты приведены в связанной с судном системе координат. \
                    Уровни отсчитываются по оси, проходящей через центр полного объема цистерны."
                )
            )
        )
    }
    ///
    /// Инициализация первой таблицы отчёта.
    ///
    /// Таблица содержит общие характеристики помещения:
    /// - район расположения;
    /// - коэффициент проницаемости;
    /// - полный объём;
    /// - максимальный момент инерции.
    ///
    /// # Параметры
    ///
    /// - `doc` — HTML-документ;
    /// - `location_area` — район расположения;
    /// - `permeability_coefficient` — коэффициент проницаемости;
    /// - `full_volume` — полный объём помещения [м³];
    /// - `max_inertia` — максимальный момент инерции IX [м⁴].
    ///
    /// # Возвращает
    ///
    /// Обновлённый [`Document`].
    fn init_table_1(
        &self,
        doc: Document,
        location_area: usize,
        permeability_coefficient: f64,
        full_volume: f64,
        max_inertia: f64,
    ) -> Document {
        doc.section(|section| section  
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
                            .text("Полный объем нетто [м³]")
                        )
                        .el(th(), |th| th
                            .text(&format!("{:.2}", permeability_coefficient*full_volume))
                        )
                    )
                    .el(tr(), |seventh_row| seventh_row
                        .el(th(), |th| th
                            .text("Максимальный момент инерции IX [м⁴]")
                        )
                        .el(th(), |th| th
                            .text(&format!("{:.2}", max_inertia))
                        )
                    )
                )
            )
        )
    }
    ///
    /// Инициализация второй таблицы отчёта.
    ///
    /// Формирует подробную таблицу заполнения помещения
    /// для каждого сочетания:
    /// - крена;
    /// - дифферента.
    ///
    /// Для каждого уровня заполнения вычисляются:
    /// - объём;
    /// - координаты центра объёма;
    /// - площадь свободной поверхности;
    /// - координаты центра площади;
    /// - моменты инерции свободной поверхности.
    ///
    /// # Параметры
    ///
    /// - `doc` — HTML-документ;
    /// - `draught_step` — шаг заполнения [м];
    /// - `report_even_keel` — предварительно рассчитанный отчёт
    ///   для случая:
    ///     - крен = 0;
    ///     - дифферент = 0;
    /// - `permeability_coefficient` — коэффициент проницаемости.
    ///
    /// # Возвращает
    ///
    /// Обновлённый [`Document`].
    fn init_table_2(
        &self,
        mut doc: Document, 
        draught_step: f64,
        report_even_keel: &Vec<(f64, OPoint<f64, Const<3>>, f64, OPoint<f64, Const<3>>, f64, f64, f64, f64)>,
        permeability_coefficient: f64,
    ) -> Document {
        for (h, t) in self.heel.iter().zip(self.trim.iter()) {
            let result = if *h == 0.0 && *t == 0.0  {
                Cow::Borrowed(report_even_keel)
            } else {
                match self.shape.read().step_displacement(*h, *t, draught_step) {
                    Ok(rep) => {
                        Cow::Owned(rep)
                    },
                    Err(e) => {
                        log::error!("Error to create report for heel={:?}, trim={:?}: {:?}", h, t, e);
                        continue;
                    }
                }
            };
            doc = doc.section(|section| {
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
                                        .text(format!("{:.2}", &h))
                                })
                                .el(th(), |th| {
                                    th
                                        .attr(colspan(), "5")
                                        .text("Дифферент [град]")
                                })
                                .el(th(), |th| { 
                                    th
                                        .text(format!("{:.2}",&t))
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
                                    th.attr(colspan(), "6")
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
                                .el(th(), |th| th.attr(colspan(), "3").text("Координаты центра площади"))
                                .el(th(), |th| th.attr(colspan(), "2").text("Момент инерции"))
                        })
                        .el(tr(), |fifth_row| {
                            fifth_row
                                .el(th(), |th| th.text("X [м]"))
                                .el(th(), |th| th.text("Y [м]"))
                                .el(th(), |th| th.text("Z [м]"))
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
                                .el(th(), |th| th.text("12"))
                        })
                    });
                    for (
                        i,
                        (
                            volume,
                            volume_center,
                            square_waterline,
                            center_waterline,
                            ix,
                            iy,
                            height,
                            mins_z,
                        ),
                    ) in result.iter().enumerate()
                    {
                        let first = i == 0;
                        let last = i + 1 == result.len();
                        let v = |s: String| if first { "-".to_string() } else { s };
                        let wl = |s: String| {
                            if first || last {
                                "-".to_string()
                            } else {
                                s
                            }
                        };
                        table = table.el(tr(), |row| {
                            row
                                .el(th(), |th| th.text(format!("{:.2}", height)))
                                .el(th(), |th| th.text(format!("{:.3}", height + mins_z)))
                                // Объём
                                .el(th(), |th| th.text(v(format!("{:.2}", volume * permeability_coefficient))))
                                // Центр объёма
                                .el(th(), |th| th.text(v(format!("{:.3}", volume_center.x))))
                                .el(th(), |th| th.text(v(format!("{:.3}", volume_center.y))))
                                .el(th(), |th| th.text(v(format!("{:.3}", volume_center.z))))
                                // Площадь
                                .el(th(), |th| th.text(wl(format!("{:.2}", square_waterline))))
                                // Центр площади
                                .el(th(), |th| th.text(wl(format!("{:.3}", center_waterline.x))))
                                .el(th(), |th| th.text(wl(format!("{:.3}", center_waterline.y))))
                                .el(th(), |th| th.text(wl(format!("{:.3}", center_waterline.z))))
                                // Моменты инерции
                                .el(th(), |th| th.text(wl(format!("{:.2}", ix))))
                                .el(th(), |th| th.text(wl(format!("{:.2}", iy))))
                        });
                    }
                    table
                })
            });
        };
        doc
    }
}
//
//
impl Eval<(), EvalResult> for RoomElementReportEval {
    ///
    /// Выполнение генерации отчёта.
    ///
    /// Последовательность работы:
    /// 1. Получение контекста результата;
    /// 2. Расчёт базового отчёта для:
    ///    - крен = 0;
    ///    - дифферент = 0;
    /// 3. Формирование HTML-документа;
    /// 4. Генерация таблиц отчёта;
    /// 5. Сохранение результата в [`RoomElementReportCtx`].
    ///
    /// # Возвращает
    ///
    /// - `Ok(EvalResult)` — отчёт успешно сформирован;
    /// - `Err(_)` — ошибка генерации отчёта.
    ///
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let id_tank = 0;
                let name_tank = String::from("Наименование танка");
                let type_tank = String::from("Тип танка");
                let location_area = 0;
                let permeability_coefficient = 0.95;
                let draught_step = 0.5;
                let report_even_keel = match self.shape.read().step_displacement(0.0, 0.0, draught_step) {
                    Ok(report) => {
                        report
                    },
                    Err(err) => return Err(err),
                };
                let result_report = self.init_table_2(
                    self.init_table_1(
                        self.init_document(
                            &self.style,
                            id_tank, 
                            name_tank,
                            type_tank,
                        ), 
                        location_area, 
                        permeability_coefficient,
                        report_even_keel.last().unwrap().0,
                        report_even_keel.iter().map(|p| p.4).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
                    ),
                    draught_step,
                    &report_even_keel,
                    permeability_coefficient,
                );
                ctx.write(
                    RoomElementReportCtx {
                        result: result_report.build(),
                    }
                )  
            }
            Err(err) => Err(err),
        }
    }
}
