use crate::{
    algorithm::eval::seakeeping::eval::{
        parametric_resonant_zone::parametric_resonant_zone_ctx::ParametricResonantZoneCtx,
        roll_frequency_eval::roll_frequency_ctx::RollingFrequencyCtx
    },
    kernel::{
        Eval, 
        types::eval_result::EvalResult
    }, 
    prelude::{ContextRead, ContextWrite}
};
///
/// Расчет [параметрической зоны резонанса бортовой качки](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#условия-возникновения-опасных-явлений)
pub struct ParametricResonantZoneEval {
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync>,
}
//
//
impl ParametricResonantZoneEval {
    ///
    /// Новый экземпляр [ParametricResonantZoneEval]
    pub fn new(ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        Self {
            ctx: Box::new(ctx),
        }
    }
}
//
//
impl Eval<(), EvalResult> for ParametricResonantZoneEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
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
            Err(err) => Err(err),
        }
    }
}
