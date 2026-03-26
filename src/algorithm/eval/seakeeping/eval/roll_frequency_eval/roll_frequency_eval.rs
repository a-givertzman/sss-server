use sal_core::dbg::Dbg;

use crate::{
    algorithm::eval::{
        parameters::ParameterID,
        seakeeping::eval::roll_frequency_eval::roll_frequency_ctx::RollingFrequencyCtx,
    },
    kernel::{Eval, types::eval_result::EvalResult},
    prelude::{ContextParamsRead, ContextWrite},
};
///
/// Расчет частоты собственных бортовых колебаний судна   
pub struct RollingFrequencyEval {
    dbg: Dbg,
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl RollingFrequencyEval {
    ///
    /// Новый экземпляр [RollingFrequencyEval]
    pub fn new(parent: impl Into<String>, ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        let dbg = Dbg::new(parent, "RollingFrequencyEval");
        Self { dbg, ctx: Box::new(ctx) }
    }
}
//
//
impl Eval<(), EvalResult> for RollingFrequencyEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let roll_period = ctx.read_params(ParameterID::RollPeriod);
                let pi = std::f64::consts::PI;
                let roll_frequency = 2.0 * pi / roll_period;
                let result = RollingFrequencyCtx { roll_frequency };
                ctx.write(result)
            }
            Err(err) => Err(err),
        }
    }
}
