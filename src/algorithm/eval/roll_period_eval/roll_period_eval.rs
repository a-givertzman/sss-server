use super::roll_period_ctx::RollingPeriodCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextRead}, eval::{parameters::ParameterID, BalanceCtx, MetacentricHeightCtx},
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, ContextWrite, CtxResult,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет периода качки судна 
pub struct RollingPeriodEval {
    dbg: Dbg,
    value: Option<RollingPeriodCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl RollingPeriodEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "RollingPeriodEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for RollingPeriodEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let metacentric_height: MetacentricHeightCtx = ctx.read();
                let balance_ctx: BalanceCtx = ctx.read();
                let length_wl = balance_ctx.length_wl;
                let breadth_wl = balance_ctx.breadth_wl;
                let mean_draught = ctx.read_params(ParameterID::DraughtMean);
                // Коэффициент для расчета периода
                let c = 0.373 + 0.023 * breadth_wl / mean_draught - 0.043 * length_wl / 100.0;
                let roll_period = if metacentric_height.h_trans_fix > 0. {
                    let h_sqrt = metacentric_height.h_trans_fix.sqrt();
                    let res = 2. * c * breadth_wl / h_sqrt;
                    log::trace!(
                        "\t RollingPeriod calculate length_wl:{length_wl} breadth_wl:{breadth_wl} mean_draught:{mean_draught} c:{c} h_sqrt: {h_sqrt} T:{res}",
                    );
                    res
                } else {
                    log::trace!("\t RollingPeriod calculate error: h_trans_fix is negative!");
                    0.
                };
                let result = RollingPeriodCtx {
                    c,
                    roll_period,
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
impl std::fmt::Debug for RollingPeriodEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RollingPeriodEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
