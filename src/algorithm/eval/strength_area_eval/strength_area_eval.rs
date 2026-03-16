use super::strength_area_ctx::StrengthAreaCtx;
use crate::{
    ContextWrite,
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{Bound, Position, data::loads::UnitCargoType},
        eval::IcingTimberCtx,
    }, 
    kernel::{eval::Eval, sync::Link, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::{query::Query, reply::Reply},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Площади боковой и горизонтальной поверхностей для расчета прочности
pub struct StrengthAreaEval {
    dbg: Dbg,
    model: Link,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl StrengthAreaEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: Link,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "StrengthAreaEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for StrengthAreaEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let unit: Vec<_> = match initial.unit.as_ref() {
                    Some(data) => data
                        .into_iter()
                        .filter(|v| v.icing_area.is_some())
                        .collect(),
                    None => return Err(error.err("Read unit error: no data!")),
                };
                let timber_unit: Vec<_> = unit
                    .iter()
                    .filter(|v| v.cargo_type == UnitCargoType::Timber)
                    .collect();
                let bounds = match initial.bounds.clone() {
                    Some(data) => data,
                    None => return Err(error.err("Read bounds error: no data!")),
                };
                // 
                // Тут все вроде правильно раскрыл,
                // Но так много действий и так сложно получается,
                // может получится хотябы часть из низ вынести в метод,
                // вроде бы действия однообразные все время должны быть
                let (const_area_v, const_area_h) = match self.model.call(Query::BoundAreas) {
                    Ok(reply) => {
                        let reply: Reply = reply;
                        match reply {
                            Reply::BoundAreas(areas) => match areas {
                                Ok(areas) => (areas.v, areas.h),
                                Err(err) => return Err(error.pass_with("Read bound_areas error", err)),
                            }
                            _ => return Err(error.err(format!("Read bound_areas - Wrong reply: {:?}", reply))),
                        }
                    }
                    Err(err) => return Err(error.pass_with("Read bound_areas error", err)),
                };
                let icing_timber_bound: IcingTimberCtx = ctx.read();
                let icing_timber_bound_x = match icing_timber_bound.bound_x() {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(
                            error.pass_with("Read icing_timber_bound_x error", err),
                        );
                    }
                };
                let icing_timber_bound_y = match icing_timber_bound.bound_y() {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(
                            error.pass_with("Read icing_timber_bound_y error", err),
                        );
                    }
                };
                // Ищем площадь парусности палубных грузов.
                // Перебираем поверхность парусности с шагом, проходим по грузам и
                // берем площадь как диапазон между максимальными ограничениями всех грузов на этом шаге.
                let mut area_v_values = Vec::new();
                let mut area_v_moment = 0.;
                // Границы грузов
                let min_x = unit
                    .iter()
                    .filter_map(|v| v.bound_x1)
                    .min_by(|a, b| a.partial_cmp(b).unwrap());
                let max_x = unit
                    .iter()
                    .filter_map(|v| v.bound_x2)
                    .max_by(|a, b| a.partial_cmp(b).unwrap());
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
                // Перебираем шпации и ищем площадь попавшую в текущую шпацию
                for (i, bound_x) in bounds.iter().enumerate() {
                    // Площадь парусности корпуса, попадающая в текущую шпацию
                    let mut current_area = match const_area_v.get(i) {
                        Some(&data) => data,
                        None => {
                            return Err(
                                error
                                    .err(format!("const_area_v.get error: no value for bound {i}")),
                            );
                        }
                    };
                    // Пересечение шпации и диапазона грузов
                    let bound_x = match bound_x.intersect(&units_bound) {
                        Ok(data) => data,
                        Err(err) => {
                            return Err(error.pass_with("bound_x.intersect error", err));
                        }
                    };
                    // Если есть пересечение шпации и диапазона грузов
                    if bound_x.is_some() {
                        // грузы имеющие площадь парусности в текущей шпации
                        let unit = unit
                            .iter()
                            .filter(|v| v.windage_area(&bound_x, &Bound::Full).unwrap_or(0.) > 0.);
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
                            current_area += bound_x.length().unwrap_or(0.) * (max_z - min_z);
                        }
                    }
                    area_v_moment += current_area * bound_x.center().unwrap_or(0.);
                    area_v_values.push(current_area);
                }
                let area_v = area_v_values.iter().sum();
                let area_v_shift = if area_v > 0. {
                    Position::new(area_v_moment / area_v, 0., 0.)
                } else {
                    Position::zero()
                };
                // Горизонтальная площадь обледенения палубного груза - леса
                let mut area_timber_h_values = Vec::new();
               // Полная горизонтальная площадь палубного груза - леса
               let mut full_area_timber_h_values = Vec::new();
               // Момент площади обледенения палубного груза - леса
                let mut area_timber_moment = 0.;
                for bound_x in bounds.iter() {
                    let mut current_area = 0.;
                    let mut full_current_area = 0.;
                    for u in &timber_unit {
                        full_current_area += u.icing_area.unwrap_or(0.);
                        current_area += match u.icing_area(
                            &bound_x
                                .intersect(&icing_timber_bound_x)
                                .unwrap_or(Bound::None),
                            &icing_timber_bound_y,
                        ) {
                            Ok(area) => area.0,
                            Err(err) => {
                                return Err(
                                    error.pass_with("Read unit horizontal_area error", err),
                                );
                            }
                        };
                    }
                    area_timber_moment += current_area * bound_x.center().unwrap_or(0.);
                    area_timber_h_values.push(current_area);
                    full_area_timber_h_values.push(full_current_area);
                }
                let area_timber_h = area_timber_h_values.iter().sum();
                let area_timber_h_shift = if area_timber_h > 0. {
                    Position::new(area_timber_moment / area_timber_h, 0., 0.)
                } else {
                    Position::zero()
                };
                // Горизонтальная площадь поверхностей корпуса
                let mut area_h_moment = 0.;
                let mut area_h_values = Vec::new();
                for (i, bound_x) in bounds.iter().enumerate() {
                    // Площадь горизонтальной поверхности корпуса, попадающая в текущую шпацию
                    // вычитаем площадь палубного груза - леса
                    let current_area = match (const_area_h.get(i), full_area_timber_h_values.get(i)) {
                        (Some(&current_const_area), Some(&current_timber_area)) => {
                            (current_const_area - current_timber_area).max(0.)
                        }
                        (Some(&current_const_area), None) => current_const_area,
                        _ => {
                            return Err(
                                error.err(format!("area_h.get error: no value for bound {i}")),
                            );
                        }
                    };
                    area_h_moment += current_area * bound_x.center().unwrap_or(0.);
                    area_h_values.push(current_area);
                }
                let area_h = area_v_values.iter().sum();
                let area_h_shift = if area_h > 0. {
                    Position::new(area_h_moment / area_h, 0., 0.)
                } else {
                    Position::zero()
                };
                let result = StrengthAreaCtx {
                    area_v,
                    area_v_shift,
                    area_v_values,
                    area_h,
                    area_h_shift,
                    area_h_values,
                    area_timber_h,
                    area_timber_h_shift,
                    area_timber_h_values,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for StrengthAreaEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StrengthAreaEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
