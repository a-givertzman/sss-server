use crate::{
    algorithm::eval::seakeeping::eval::roll_frequency_eval::roll_frequency_ctx::RollingFrequencyCtx, 
    kernel::{
        Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::{
        ContextReadRef, ContextWrite, InitialCtx
    }
};
///
/// Расчет частоты собственных бортовых колебаний судна   
pub struct RollingFrequencyEval {
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl RollingFrequencyEval {
    ///
    /// Новый экземпляр [RollingFrequencyEval]
    pub fn new(
        ctx: impl Eval<(), EvalResult> + Send + Sync + 'static,
    ) -> Self {
        Self { ctx: Box::new(ctx) }
    }
}
//
//
impl Eval<(), EvalResult> for RollingFrequencyEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let roll_period = ContextReadRef::<InitialCtx>::read_ref(&ctx).rolling_period;
                let pi = std::f64::consts::PI;
                let roll_frequency = 2.0 * pi / roll_period;
                let result = RollingFrequencyCtx { roll_frequency };
                ctx.write(result)
            }
            Err(err) => Err(err),
        }
    }
}
