use super::criterion_stability_ctx::CriterionStabilityCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef}, eval::MetacentricHeightCtx,
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ContextWrite, CtxResult,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет периода качки судна 
pub struct CriterionStabilityEval {
    dbg: Dbg,
    value: Option<CriterionStabilityCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl CriterionStabilityEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "CriterionStabilityEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for CriterionStabilityEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                CtxResult::Err(error.pass_with("Not implemented yet"))
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for CriterionStabilityEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CriterionStabilityEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
