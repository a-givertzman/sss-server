use super::wind_ctx::WindCtx;
use crate::{
    algorithm::{
        context::context_access::{ContextRead, ContextReadRef},
        entities::{data::loads::UnitCargoType, parameters::{IParameters, Parameters}, Bound, Moment, Position},
        eval::IcingTimberCtx,
    }, kernel::{eval::Eval, types::eval_result::EvalResult}, prelude::InitialCtx, ship_model::model_link::{IModelLink, ModelLink}, ContextWrite, CtxResult
};
use sal_core::{dbg::Dbg, error::Error};
///
/// Расчет плеча кренящего момента от давления ветра
pub struct WindEval {
    dbg: Dbg,
    value: Option<WindCtx>,
    ctx: Box<dyn Eval<(), EvalResult>>,
}
//
//
impl WindEval {
    ///
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<(), EvalResult> + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "WindEval");
        Self {
            dbg,
            value: None,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for WindEval {
    fn eval(&mut self, _: ()) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            CtxResult::Ok(ctx) => {
                let initial: &InitialCtx = ctx.read_ref();
                let parameters: Parameters = ctx.read(); 

                let result = WindCtx {
                    arm_wind_static: todo!(),
                    arm_wind_dynamic: todo!(),
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
impl std::fmt::Debug for WindEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindEval")
            .field("dbg", &self.dbg)
            .field("value", &self.value)
            .finish()
    }
}
