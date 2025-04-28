use super::metacentric_height_ctx::MetacentricHeightCtx;
use crate::{
    ContextWrite, CtxResult,
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::data::{loads::UnitCargoType, ship_type::ShipType},
        eval::{CriterionData, CriterionID, LoadsCtx},
    },
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия метацентрической высоты
pub struct MetacentricHeightEval {
    dbg: Dbg,
    value: Option<MetacentricHeightCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl MetacentricHeightEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "MetacentricHeightEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for MetacentricHeightEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let metacentric_height: crate::algorithm::eval::metacentric_height_eval::metacentric_height_ctx::MetacentricHeightCtx = ctx.read();
                let loads: LoadsCtx = ctx.read();
                let ship = initial
                    .ship
                    .as_ref()
                    .expect("MetacentricHeightEval eval error: no ship!");
                let have_grain = !loads.bulk.is_empty();
                let unit: Vec<_> = match initial.unit.as_ref() {
                    Some(data) => data
                        .into_iter()
                        .filter(|v| v.icing_area.is_some())
                        .collect(),
                    None => return CtxResult::Err(error.err("Read unit error: no data!")),
                };
                let have_timber = unit.iter().any(|v| v.cargo_type == UnitCargoType::Timber);
                let ship_type = match ShipType::from_str(&ship.ship_type) {
                    Ok(ship_type) => ship_type,
                    Err(err) => {
                        let error = error.pass_with("ShipType::from_str", err);
                        log::error!("{}", error);
                        let result = CriterionData::new_error(
                            CriterionID::MinMetacentricHight,
                            "Ошибка расчета площади под положительной частью диаграммы статической остойчивости 0-40 градусов: ".to_owned() + &error.to_string(),
                        );
                        let result = MetacentricHeightCtx { data: result };
                        self.value = Some(result.clone());
                        return ctx.write(result);
                    }
                };
                // Все суда
                let target = if have_grain {
                    0.3
                } else if ship_type == ShipType::RoRo {
                    0.2
                } else if have_timber {
                    0.1
                } else {
                    0.15
                };
                let result = MetacentricHeightCtx {
                    data: CriterionData::new_result(
                        CriterionID::MinMetacentricHight,
                        metacentric_height.h_trans_fix,
                        target,
                    ),
                };
                self.value = Some(result.clone());
                ctx.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for MetacentricHeightEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetacentricHeightEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
