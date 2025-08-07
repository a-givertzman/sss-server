use crate::{
    algorithm::{
        context::context_access::ContextRead, 
        eval::{
            roll_frequency_eval::roll_frequency_ctx::RollingFrequencyCtx, zg_eval::Zg, RollingPeriodCtx
        },
    }, 
    kernel::{
        eval::Eval, 
        types::eval_result::EvalResult
    }, 
    ContextWrite,
};
use sal_core::{
    dbg::Dbg, 
    error::Error
};
///
/// Расчет частоты собственных бортовых колебаний судна   
pub struct RollingFrequencyEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<Zg, EvalResult> + Send + Sync>,
}
//
//
impl RollingFrequencyEval {
    ///
    /// Новый экземпляр [RollingFrequencyEval]
    pub fn new(
        parent: impl Into<String>,
        ctx: impl Eval<Zg, EvalResult> + Send + Sync + 'static,
    ) -> Self {
        let dbg = Dbg::new(parent, "RollingFrequencyEval");
        Self {
            dbg,
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<Zg, EvalResult> for RollingFrequencyEval {
    fn eval(&self, z_g_fix: Zg) -> EvalResult {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(z_g_fix) {
            Ok(ctx) => {
                let roll_period = ContextRead::<RollingPeriodCtx>::read(&ctx).roll_period.clone();
                let pi = std::f64::consts::PI;
                let roll_frequency = 2.0 * pi / roll_period;
                let result = RollingFrequencyCtx {
                    roll_frequency,
                }; 
                ctx.write(result)
            }
            Err(err) => Err(error.pass_with("Read context error", err)),
        }
    }
}
//
//
impl std::fmt::Debug for RollingFrequencyEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RollingFrequencyEval")
            .field("dbg", &self.dbg)
            .finish()
    }
}
