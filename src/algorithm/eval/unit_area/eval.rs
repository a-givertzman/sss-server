use crate::algorithm::entities::Moment;
use crate::algorithm::eval::UnitAreaCtx;
use crate::kernel::Eval;
use crate::{
    algorithm::{context::context_access::ContextReadRef, entities::Bound},
    kernel::types::eval_result::EvalResult,
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Площади боковой поверхности грузов
pub struct UnitAreaEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl UnitAreaEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "UnitAreaEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for UnitAreaEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let unit = match initial.unit.as_ref() {
                    Some(data) => data,
                    None => return Err(error.err("Read unit error: no data!")),
                };
                let bounds = match initial.bounds.clone() {
                    Some(data) => data,
                    None => return Err(error.err("Read bounds error: no data!")),
                };
                // Границы грузов
                let min_x = unit
                    .iter()
                    .filter_map(|v| v.bound_x1)
                    .min_by(|a, b| a.partial_cmp(&b).unwrap());
                let max_x = unit
                    .iter()
                    .filter_map(|v| v.bound_x2)
                    .max_by(|a, b| a.partial_cmp(&b).unwrap());
                // Если есть границы грузов ищем распределения площадей грузов
                let units_bound = if let (Some(min_x), Some(max_x)) = (min_x, max_x) {
                    // Диапазон грузов по оси Х
                    match Bound::new(min_x, max_x) {
                        Ok(data) => data,
                        Err(err) => {
                            return Err(error.pass_with("units_bound error", err));
                        }
                    }
                } else {
                    Bound::None
                };
                // Ищем площадь парусности палубных грузов.
                // Перебираем поверхность парусности с шагом, проходим по грузам и
                // берем площадь как диапазон между максимальными ограничениями всех грузов на этом шаге.
                let (av_dc, mvx_dc, mvz_dc, distr_v ) = {
                    let mut av_dc = 0.; // Площадь парусности палубного груза, м^2
                    let (mut mvx_dc, mut mvz_dc) = (0., 0.); // Cтатический момент площади парусности палубного груза, м^2
                    let mut distr_v = Vec::new();  // Распределение площади парусности палубного груза по шпациям, м^2
                    let unit: Vec<_> = unit.iter().filter(|v| v.windage_area.is_some()).collect();
                    // Перебираем шпации и ищем площадь попавшую в текущую шпацию
                    for (_i, bound_x) in bounds.iter().enumerate() {
                        let mut current_area = 0.;
                        let mut current_moment_x = 0.;
                        let mut current_moment_z = 0.;
                        // Пересечение шпации и диапазона грузов
                        let bound_x = match bound_x.intersect(&units_bound) {
                            Ok(bound) => bound,
                            Err(err) => {
                                return Err(error.pass_with("bound_x.intersect error", err));
                            }
                        };
                        // Если есть пересечение шпации и диапазона грузов
                        if bound_x.is_some() {
                            assert!(bound_x.is_value());
                            // грузы имеющие площадь парусности в текущей шпации
                            let unit =
                                unit.iter()
                                    .filter(|v| {
                                        v.windage_area(&bound_x, &Bound::Full).unwrap_or(0.) > 0.
                                    });
                            // границы грузов в текущей шпации
                            let min_z = unit
                                .clone()
                                .filter_map(|v| v.bound_z1)
                                .min_by(|&a, &b| a.partial_cmp(&b).unwrap());
                            let max_z = unit
                                .filter_map(|v| v.bound_z2)
                                .max_by(|&a, &b| a.partial_cmp(&b).unwrap());
                            // Прибавляем к площади прямоугольник площади грузов
                            if let (Some(min_z), Some(max_z)) = (min_z, max_z) {
                                let z = min_z;
                                let dx = bound_x.length().unwrap_or(0.);
                                let dz = max_z - min_z;
                                let unit_area = dx * dz;
                                current_area += unit_area;
                                current_moment_x += bound_x.center().unwrap_or(0.);
                                current_moment_z += (z + dz / 2.) * unit_area;
                            }
                        }
                        av_dc += current_area;
                        mvx_dc += current_moment_x;
                        mvz_dc += current_moment_z;
                        distr_v.push(current_area);
                    }
                    (av_dc, mvx_dc, mvz_dc, distr_v)
                };
                // Суммарное изменение момента площади горизонтальных поверхностей относительно палубы
                // Считаем как произведение площади груза на его высоту
                let delta_moment_h = {
                    let unit: Vec<_> = unit.iter().filter(|v| v.icing_area.is_some()).collect();
                    let mut delta_moment_h = Moment::zero();
                    for u in &unit {
                        match u.icing_area(
                            &Bound::Full,
                            &Bound::Full,
                        ) {
                            Ok((_, _, current)) => {
                                delta_moment_h += current;
                            }
                            Err(err) => {
                                return Err(error.pass_with("u.icing_area", err));
                            }
                        };
                    }
                    delta_moment_h
                };
                log::info!(
                    "UnitArea av_dc:{:.3} mvx_dc:{:.3} mvz_dc:{:.3} delta_moment_h:{:.3}",
                    av_dc,
                    mvx_dc,
                    mvz_dc,
                    delta_moment_h,
                );
                let result = UnitAreaCtx {
                    av_dc,
                    mv_dc: Moment::new(mvx_dc, 0., mvz_dc),
                    delta_moment_h,
                    distr_v,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for UnitAreaEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnitAreaCtx")
            .field("dbg", &self.dbg)
            .finish()
    }
}
