use crate::algorithm::context::context_access::{ContextRead, ContextReadRef};
use crate::algorithm::eval::ReserveBuoyncyCtx;
use crate::algorithm::eval::{CriterionData, CriterionID, StabilityBalanceCtx};
use crate::prelude::InitialCtx;
use crate::{
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::*,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия запаса плавучести в носу
pub struct ReserveBuoyncyEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl ReserveBuoyncyEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "ReserveBuoyncyEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for ReserveBuoyncyEval {
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let balance: StabilityBalanceCtx = ctx.read();
                let ship_parameters = initial.ship_parameters.as_ref().unwrap();
                let bow_area_min = *ship_parameters
                    .get("Calculated minimum bow area")
                    .ok_or(error.err("No bow_area_min"))?;
                log::info!(
                    "Criterion ReserveBuoyncyInBow bow_area:{:.3} bow_area_min:{:.3}",
                    balance.bow_area,
                    bow_area_min
                );
                let result = ReserveBuoyncyCtx {
                    data: CriterionData::new_result(
                        CriterionID::ReserveBuoyncyInBow,
                        balance.bow_area,
                        bow_area_min,
                    ),
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ReserveBuoyncyEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReserveBuoyncyEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
