use crate::algorithm::eval::zg_eval::Zg;
use crate::algorithm::eval::{
    ParametricResonantZoneCtx, 
    RollingFrequencyCtx
};
use crate::{
    ContextWrite,
    algorithm::context::context_access::ContextRead,
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    },
};
use sal_core::{
    dbg::Dbg, 
    error::Error
};
///
/// Расчет параметрической зоны резонанса бортовой качки
pub struct ParametricResonantZoneEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl ParametricResonantZoneEval {
    ///
    pub fn new(parent: impl Into<String>, ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "ParametricResonantZoneEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for ParametricResonantZoneEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let roll_frequency = ContextRead::<RollingFrequencyCtx>::read(&ctx).roll_frequency.clone();
                let left_side = 1.9 * roll_frequency;
                let right_side = 2.1 * roll_frequency;
                let result = ParametricResonantZoneCtx {
                    left_side,
                    right_side,
                };
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for ParametricResonantZoneEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParametricResonantZoneEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
