use super::strength_area_ctx::StrengthAreaCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{data::loads::UnitCargoType, Bound},
        eval::IcingTimberCtx,
    },
    kernel::{dbgid::dbgid::DbgId, eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
    ship_model::model_link::ModelLink,
    ContextWrite, CtxResult,
};
use sal_sync::services::entity::error::str_err::StrErr;
///
/// Площади боковой и горизонтальной поверхностей для расчета прочности
pub struct StrengthAreaEval {
    dbg: DbgId,
    model: ModelLink,
    value: Option<StrengthAreaCtx>,
    ctx: Box<dyn Eval<(), EvalResult> + Send>,
}
//
//
impl StrengthAreaEval {
    ///
    /// Fetches all initiall data
    /// - 'api_client' - access to the database
    pub fn new(
        parent: impl Into<String>,
        model: ModelLink,
        ctx: impl Eval<(), EvalResult> + Send + 'static,
    ) -> Self {
        let dbg = DbgId::with_parent(&DbgId(parent.into()), "BoundArea");
        Self {
            dbg,
            model,
            value: None,
            ctx: Box::new(ctx),
        }
    }
    //
    //
}
impl Eval<(), EvalResult> for StrengthAreaEval {
    fn eval(&mut self, _: ()) -> futures::future::BoxFuture<'_, EvalResult> {
        Box::pin(async move {
            match self.ctx.eval(()).await {
                CtxResult::Ok(ctx) => {
                    let initial: &InitialCtx = ctx.read_ref();
                    let unit = match initial.unit.clone() {
                        Some(data) => data.data().iter().filter(|v| v.icing_area.is_some()),
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read unit error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let timber_unit = unit.filter(|v| v.cargo_type == UnitCargoType::Timber);
                    let bounds = match initial.bounds.clone() {
                        Some(data) => data,
                        None => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read bounds error: no data!",
                                self.dbg
                            )))
                        }
                    };
                    let (const_area_v, const_area_h) = match self.model.bound_areas().await {
                        Ok((area_v, area_h)) => (area_v, area_h),
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read bound_areas error: {:?}",
                                self.dbg, err
                            )));
                        }
                    };
                    let icing_timber_bound: IcingTimberCtx = ctx.read();
                    let icing_timber_bound_x = match icing_timber_bound.bound_x() {
                        Ok(data) => data,
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_timber_bound_x error: {:?}",
                                self.dbg, err
                            )));
                        }
                    };
                    let icing_timber_bound_y = match icing_timber_bound.bound_y() {
                        Ok(data) => data,
                        Err(err) => {
                            return CtxResult::Err(StrErr(format!(
                                "{}.eval | Read icing_timber_bound_y error: {:?}",
                                self.dbg, err
                            )));
                        }
                    };
                    let mut area_h = Vec::new();
                    let mut area_timber_h = Vec::new();

                    // Ищем площадь горизонтальной поверхности палубных грузов.
                    // Перебираем горизонтальую поверхность с шагом, проходим по грузам и
                    // берем максимальную площадь среди всех грузов на этом шаге.
                    let min_x = unit
                        .filter_map(|v| v.bound_x1)
                        .min_by(|&a, &b| a.partial_cmp(&b).unwrap());
                    let max_x = unit
                        .filter_map(|v| v.bound_x2)
                        .max_by(|&a, &b| a.partial_cmp(&b).unwrap());
                    let units_bound = if let (Some(min_x), Some(max_x)) = (min_x, max_x) { 
                        Bound::new(min_x, max_x)?;
                    }
                    else {

                    };
                    
                    for b in bounds.iter() {
                        if let (Some(min_x), Some(max_x)) = (min_x, max_x) {
                            let mut area_sum = 0.;
                            let bound = b.intersect(&Bound::new(min_x, max_x)?)?;

                            if bound.is_some() {
                                let (min_x, max_x) = (
                                    match bound.start() {
                                        Some(data) => data,
                                        None => {
                                            return CtxResult::Err(StrErr(format!(
                                                "{}.eval | bound intersect error: {:?}",
                                                self.dbg, err
                                            )));
                                        }
                                    },

                                    bound.start().ok_or(Error::FromString(
                                        "Area area_v error: bound.start()".to_owned(),
                                    ))?,
                                    bound.end().ok_or(Error::FromString(
                                        "Area area_v error: bound.end()".to_owned(),
                                    ))?,
                                );
                                area_sum += Bounds::from_min_max(min_x, max_x, 200)?
                                    .iter()
                                    .map(|bound_x| {
                                        let min_z = self
                                            .desks
                                            .iter()
                                            .filter_map(|v| v.min_z())
                                            .min_by(|&a, &b| a.partial_cmp(&b).unwrap());
                                        let max_z = self
                                            .desks
                                            .iter()
                                            .filter_map(|v| v.max_z())
                                            .max_by(|&a, &b| a.partial_cmp(&b).unwrap());
                                        if let (Some(min_z), Some(max_z)) = (min_z, max_z) {
                                            Bounds::from_min_max(min_z, max_z, 50)
                                                .expect("Area area_v error: Bounds::from_min_max")
                                                .iter()
                                                .map(|bound_z| {
                                                    self.desks
                                                        .iter()
                                                        .filter_map(|v| {
                                                            v.windage_area(bound_x, bound_z).ok()
                                                        })
                                                        .max_by(|&a, &b| a.partial_cmp(&b).unwrap())
                                                        .unwrap_or(0.)
                                                })
                                                .sum()
                                        } else {
                                            0.
                                        }
                                    })
                                    .sum::<f64>()
                            }

                            area_h.push(area_sum);
                        }

                        {
                            let mut area_sum = 0.;
                            for u in timber_unit {
                                area_sum += match u.horizontal_area(
                                    &b.intersect(&icing_timber_bound_x).unwrap_or(Bound::None),
                                    &icing_timber_bound_y,
                                ) {
                                    Ok(area) => area,
                                    Err(err) => {
                                        return CtxResult::Err(StrErr(format!(
                                            "{}.eval | Read unit horizontal_area error: {:?}",
                                            self.dbg, err
                                        )))
                                    }
                                };
                            }
                            area_timber_h.push(area_sum);
                        }
                    }

                    for v in const_area_v.iter() {
                        area_sum += v.value(bound)?;
                    }

                    let result = StrengthAreaCtx {
                        area_v,
                        area_h: const_area_h,
                        area_timber_h,
                    };
                    self.value = Some(result.clone());
                    ctx.write(result)
                }
                CtxResult::Err(err) => CtxResult::Err(StrErr(format!(
                    "{}.eval | Read context error: {:?}",
                    self.dbg, err
                ))),
                CtxResult::None => CtxResult::None,
            }
        })
    }
}
//
//
impl std::fmt::Debug for StrengthAreaEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundArea")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
