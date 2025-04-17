use super::windage_ctx::WindageCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{data::loads::UnitCargoType, parameters::{IParameters, Parameters}, Bound, Moment, Position},
        eval::IcingTimberCtx,
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::model_link::{IModelLink, ModelLink}, ContextWrite, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Парусность судна, площадь и положение
/// центра относительно миделя и ОП
pub struct WindageEval {
    dbg: Dbg,
    value: Option<WindageCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl WindageEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "WindageEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for WindageEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let parameters: Parameters = ctx.read(); 

                let result = WindageCtx {
                    a_v: todo!(),
                    z_v: todo!(),
                };
                self.value = Some(result.clone());
                ctx.write(parameters)?;
                ctx.write(result)
            }
            CtxResult::Err(err) => CtxResult::Err(error.pass_with("Read context error", err)),
            CtxResult::None => CtxResult::None,
        }
    }
}
//
//
impl std::fmt::Debug for WindageEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindageEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
