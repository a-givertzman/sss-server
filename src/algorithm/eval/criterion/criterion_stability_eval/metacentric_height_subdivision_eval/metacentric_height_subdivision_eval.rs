use super::metacentric_height_subdivision_ctx::MetacentricHeightSubdivisionCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextParamsRead, ContextRead, ContextReadRef},
        eval::{parameters::ParameterID, BalanceCtx, CriterionData, CriterionID, MetacentricHeightCtx},
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ContextWrite, CtxResult
};
use crate::algorithm::entities::{Curve, ICurve};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия метацентрической высоты
pub struct MetacentricHeightSubdivisionEval {
    dbg: Dbg,
    value: Option<MetacentricHeightSubdivisionCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl MetacentricHeightSubdivisionEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "MetacentricHeightSubdivisionEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for MetacentricHeightSubdivisionEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let metacentric_height: MetacentricHeightCtx = ctx.read();
                let data = initial.h_subdivision.as_ref().unwrap();
                let mean_draught = ctx.read_params(ParameterID::DraughtMean);
                let h_subdivision = Curve::new_linear(data)
                    .map_err(|err| error.pass_with("h_subdivision Curve::new_linear", err))?
                    .value(mean_draught)
                    .map_err(|err| error.pass_with("h_subdivision Curve::value", err))?;
                let result = MetacentricHeightSubdivisionCtx {
                    data: CriterionData::new_result(
                        CriterionID::MinMetacentricHeightSubdivIndex,
                        metacentric_height.h_trans_fix,
                        h_subdivision,
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
impl std::fmt::Debug for MetacentricHeightSubdivisionEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetacentricHeightSubdivisionEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
