use super::trim_ctx::TrimCtx;
use crate::algorithm::context::context_access::ContextParamsRead;
use crate::algorithm::eval::parameters::ParameterID;
use crate::algorithm::eval::{CriterionData, CriterionID};
use crate::{
    ContextWrite,
    algorithm::context::context_access::ContextReadRef,
    kernel::{eval::Eval, types::eval_result::EvalResult},
    prelude::InitialCtx,
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет критерия максимального и минимального дифферента
pub struct TrimEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl TrimEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + 'static) -> Self {
        let dbg = Dbg::new(parent, "TrimEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for TrimEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let draught_bow = ctx.read_params(ParameterID::DraughtBow);    
                let draught_stern = ctx.read_params(ParameterID::DraughtStern);  
                let forward_trim = CriterionData::new_result(
                        CriterionID::MaximumForwardTrim,
                        draught_bow,
                        self.forward_trim,
                    );
                let aft_trim = CriterionData::new_result(CriterionID::MaximumAftTrim, draught_stern, self.aft_trim);
                let result = TrimCtx {
                    data: vec![aft_trim, forward_trim],
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for TrimEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrimEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
