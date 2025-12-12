use crate::algorithm::eval::StabilityAreaCtx;
use crate::algorithm::eval::parameters::ParameterID;
use crate::kernel::Eval;
use crate::prelude::ContextParamsRead;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{
            Bound, Moment, Position,
            data::loads::UnitCargoType,
            ship_model::{StabilityArea, ship_model::ShipModel},
        },
        eval::IcingTimberCtx,
    },
    kernel::{
        types::{Arc, eval_result::EvalResult},
    },
    prelude::{ContextWrite, InitialCtx},
};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::sync::RwLock;
///
/// Площади боковой и горизонтальной поверхностей для расчета остойчивости
pub struct StabilityAreaEval {
    dbg: Dbg,
    model: Arc<RwLock<ShipModel>>,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl StabilityAreaEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        model: Arc<RwLock<ShipModel>>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "StabilityAreaEval");
        Self {
            dbg,
            model,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for StabilityAreaEval {
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
                let draught = ctx.read_params(ParameterID::DraughtMid);
                let StabilityArea {
                    area_windage,
                    area_windage_z,
                    area_horisontal,
                    area_horisontal_z,                  
                    area_volume_z,
                    delta_area_windage,
                } = match self.model.read().stability_area(draught) {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(error.pass_with("model.stability_area", err));
                    }
                };
                let icing_timber_bound: IcingTimberCtx = ctx.read();
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
                let mut area_v = area_windage; // Площадь парусности корпуса
                let mut moment_v = area_windage*area_windage_z;// Момент парусности корпуса
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
                // Перебираем шпации и ищем площадь попавшую в текущую шпацию
                for (_i, bound_x) in bounds.iter().enumerate() {
                    let mut current_area = 0.;
                    let mut current_moment = 0.;
                    // Пересечение шпации и диапазона грузов
                    let bound_x = match bound_x.intersect(&units_bound) {
                        Ok(bound) => bound,
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
                          //  let x = bound_x.center().unwrap_or(0.);
                            let z = min_z;
                            let dx = bound_x.length().unwrap_or(0.);
                            let dz = max_z - min_z;
                            let unit_area = dx * dz;
                            current_area += unit_area;
                            current_moment += (z + dz / 2.)*unit_area;
                        }
                    }
                    area_v += current_area;
                    moment_v += current_moment;
                }
                // Горизонтальная площадь поверхностей корпуса
                let mut moment_h = area_horisontal*area_horisontal_z;
                // Момент горизонтальной площади обледенения палубного груза - леса
                let mut moment_timber_h = 0.;
                // Изменение момента горизонтальной площади обледенения палубного груза - леса
                // относительно палубы
                let mut delta_moment_timber_h = 0.;
                for bound_x in bounds.iter() {
                    let mut current_moment = 0.;
                    let mut current_delta_moment = 0.;
                    for u in &timber_unit {
                        let current_bound_x = bound_x
                            .intersect(&icing_timber_bound_x)
                            .unwrap_or(Bound::None);
                        match u.icing_area(&current_bound_x, &icing_timber_bound_y) {
                            Ok((_, full_moment, delta_moment)) => {
                                // let x = current_bound_x.center().unwrap_or(0.);
                                // let z = u.centre_of_icing_area.unwrap_or(Position::zero()).z();
                                current_moment += full_moment.z();
                                current_delta_moment += delta_moment.z();
                            }
                            Err(err) => {
                                return Err(error.pass_with("Read unit horizontal_area error", err));
                            }
                        };
                    }
                    moment_timber_h += current_moment;
                    delta_moment_timber_h += current_delta_moment;
                }
                let result = StabilityAreaCtx {
                    area_v,
                    moment_v,
                    moment_h,
                    moment_timber_h,
                    delta_moment_timber_h,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for StabilityAreaEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StabilityAreaEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
