use super::reserve_buoyncy_ctx::ReserveBuoyncyCtx;
use crate::algorithm::context::context_access::{ContextRead, ContextReadRef};
use crate::algorithm::eval::{BalanceCtx, CriterionData, CriterionID};
use crate::prelude::InitialCtx;
use crate::{
    ContextWrite, CtxResult,
    kernel::{eval::Eval, types::eval_result::EvalResult},
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия запаса плавучести в носу
pub struct ReserveBuoyncyEval {
    dbg: Dbg,
    value: Option<ReserveBuoyncyCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl ReserveBuoyncyEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "ReserveBuoyncyEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for ReserveBuoyncyEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let balance: BalanceCtx = ctx.read();               
                let ship_parameters = initial.ship_parameters.as_ref().unwrap();
                let bow_area_min = *ship_parameters
                    .get("Calculated minimum bow area")
                    .ok_or(error.err("No bow_area_min"))?;
                let result = ReserveBuoyncyCtx {
                    data: CriterionData::new_result(
                        CriterionID::ReserveBuoyncyInBow,
                        balance.bow_area,
                        bow_area_min,
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
impl std::fmt::Debug for ReserveBuoyncyEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReserveBuoyncyEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
