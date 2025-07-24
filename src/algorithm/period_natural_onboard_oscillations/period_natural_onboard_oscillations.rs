use sal_core::{
    dbg::Dbg, 
    error::Error
};
use crate::{
    algorithm::{
        context::context_access::{
        ContextReadRef
    }, 
    period_natural_onboard_oscillations::PeriodNaturalOnBoardOscillationsCtx
}, 
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::{ContextWrite, InitialCtx},
};
///
/// Calculating [period of natural on board oscillations](https://github.com/a-givertzman/sss-server/issues/83)
pub struct PeriodNaturalOnBoardOscillations {
    dbg: Dbg,
    // period
    result: Option<PeriodNaturalOnBoardOscillationsCtx>,
    // [context](src/algorithm/context/context.rs)
    ctx: Box<dyn Eval<(), EvalResult> + Send>

}
//
//
impl PeriodNaturalOnBoardOscillations {
    ///
    /// New instance [PeriodNaturalOnBoardOscillations]
    pub fn new(ctx: impl Eval<(), EvalResult> + Send + 'static) -> Self {
        Self {
            dbg: Dbg::own("PeriodNaturalOnBoardOscillations"),
            result: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for PeriodNaturalOnBoardOscillations {
    ///
    /// Calculating period of natural onboard oscillations
    fn eval(&self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let initial_ctx = ContextReadRef::<InitialCtx>::read_ref(&ctx);
                let length_lbp = initial_ctx.length_lbp;
                let b = initial_ctx.b;
                let h = initial_ctx.h;
                let d = initial_ctx.d;
                let c = 0.373 + 0.023 * b / d - 0.043 * length_lbp / 100.0;
                let result = 2.0 * c * b / h.sqrt();
                ctx.write(
                    PeriodNaturalOnBoardOscillationsCtx {
                        result,
                    }
                )
            },
            Err(err) => Err(error.pass(err)),
        }
    }
}