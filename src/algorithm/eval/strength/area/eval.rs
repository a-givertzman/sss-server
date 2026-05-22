use sal_3dlib_core::math::*;
use crate::algorithm::eval::icing_timber_bound::ctx::IcingTimberBoundCtx;
use crate::algorithm::eval::strength::AreaStrCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{
            data::loads::UnitCargoType, ship_model::ship_model::ShipModel,
        },
    },
    kernel::{
        Eval,
        types::{Arc, RwLock, eval_result::EvalResult},
    },
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Площади боковой и горизонтальной поверхностей для расчета прочности
pub struct AreaStrEval {
    dbg: Dbg,
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl AreaStrEval {
    //
    pub fn new(
        parent: impl Into<String>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "AreaStrEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for AreaStrEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let unit: Vec<_> = match initial.unit.as_ref() {
                    Some(data) => data
                        .iter()
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
                let (str_area_v, str_area_h) = match self.model.read().strength_area() {
                    Ok(areas) => (areas.v, areas.h),
                    Err(err) => return Err(error.pass_with("model.strength_area", err)),
                };
                let icing_timber_bound: IcingTimberBoundCtx = ctx.read();
                let icing_timber_bound_x = match icing_timber_bound.bound_x() {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(error.pass_with("Read icing_timber_bound_x error", err));
                    }
                };
                let icing_timber_bound_y = match icing_timber_bound.bound_y() {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(error.pass_with("Read icing_timber_bound_y error", err));
                    }
                };
                // Ищем площадь парусности палубных грузов.
                // Перебираем поверхность парусности с шагом, проходим по грузам и
                // берем площадь как диапазон между максимальными ограничениями всех грузов на этом шаге.
                // Площадь парусности считаем как сумму площадей по шпациям
                let mut area_v = Vec::new();
                // Границы грузов
                let min_x = unit
                    .iter()
                    .filter_map(|v| v.bound_x1)
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let max_x = unit
                    .iter()
                    .filter_map(|v| v.bound_x2)
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
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
                    let mut current_area = match str_area_v.get(i) {
                        Some(&data) => data,
                        None => {
                            return Err(error
                                .err(format!("const_area_v.get error: no value for bound {i}")));
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
                            .min_by(|&a, &b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal));
                        let max_z = unit
                            .filter_map(|v| v.bound_z2)
                            .max_by(|&a, &b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal));
                        // Прибавляем к площади прямоугольник площади грузов
                        if let (Some(min_z), Some(max_z)) = (min_z, max_z) {
                            current_area += bound_x.length().unwrap_or(0.) * (max_z - min_z);
                        }
                    }
                    area_v.push(current_area);
                }
                // Горизонтальная площадь обледенения палубного груза - леса
                let mut timber_icing_h = Vec::new();
                // Полная горизонтальная площадь палубного груза - леса
                let mut full_timber_icing_h_values = Vec::new();
                // Момент площади обледенения палубного груза - леса
                let mut icing_timber_moment = Moment::zero();
                // Изменение момента горизонтальной площади обледенения палубного груза - леса
                // относительно палубы
                let mut icing_delta_timber_moment = Moment::zero();            
                for bound_x in bounds.iter() {
                    let mut icing_current_area = 0.;
                    let mut full_icing_current_area = 0.;
                    for u in &timber_unit {
                        let u_icing_area = match u.icing_area(bound_x, &Bound::Full) {
                            Ok((area, ..)) => area,
                            Err(err) => {
                                log::error!("{}", error.pass_with("Read unit horizontal_area error", err));
                                continue;
                            },
                        };
                        full_icing_current_area += u_icing_area;
                        match u.icing_area(
                            &bound_x
                                .intersect(&icing_timber_bound_x)
                                .unwrap_or(Bound::None),
                            &icing_timber_bound_y,
                        ) {
                            Ok((area, moment, delta_moment)) => {
                                icing_current_area += area;
                                icing_timber_moment += moment;
                                icing_delta_timber_moment += delta_moment;
                            }
                            Err(err) => {
                                log::error!("{}", error.pass_with("Read unit horizontal_area error", err));
                                continue;
                            },
                        };
                    }
                    timber_icing_h.push(icing_current_area);
                    full_timber_icing_h_values.push(full_icing_current_area);
                }
                // Горизонтальная площадь поверхностей корпуса
                let mut area_h = Vec::new();
                for (i, _) in bounds.iter().enumerate() {
                    // Площадь горизонтальной поверхности корпуса, попадающая в текущую шпацию
                    // вычитаем площадь палубного груза - леса
                    let current_area = match (str_area_h.get(i), full_timber_icing_h_values.get(i))
                    {
                        (Some(&current_const_area), Some(&current_timber_area)) => {
                            (current_const_area - current_timber_area).max(0.)
                        }
                        (Some(&current_const_area), None) => current_const_area,
                        _ => {
                            return Err(
                                error.err(format!("area_h.get error: no value for bound {i}"))
                            );
                        }
                    };
                    area_h.push(current_area);
                }
                let result = AreaStrCtx {
                    area_v,
                    area_h,
                    timber_icing_h,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for AreaStrEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AreaStrEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
