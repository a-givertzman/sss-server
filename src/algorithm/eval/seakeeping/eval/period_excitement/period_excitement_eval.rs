use crate::{
    algorithm::eval::seakeeping::eval::period_excitement::period_excitement_ctx::PeriodExcitementCtx, 
    kernel::{
        Eval, types::eval_result::EvalResult
    }, 
    prelude::{
        ContextReadRef, ContextWrite, InitialCtx
    }
};
///
/// Расчет [периода волнения](https://github.com/a-givertzman/sss/blob/50-guidance-to-the-master-according-to-msc1-circ1228/design/algorithm/part06_seakeeping/part06_seakeeping.md#порядок-расчета)
pub struct PeriodExcitementEval {
    ctx: Box<dyn Eval<(), EvalResult> + Send + Sync + 'static>,
}
//
//
impl PeriodExcitementEval {
    ///
    /// Новый экземпляр [PeriodExcitementEval]
    pub fn new(ctx: impl Eval<(), EvalResult> + Send + Sync + 'static) -> Self {
        Self { ctx: Box::new(ctx) }
    }
}
//
//
impl Eval<(), EvalResult> for PeriodExcitementEval {
    fn eval(&self, _: ()) -> EvalResult {
        match self.ctx.eval(()) {
            Ok(ctx) => {
                let length_wave = ContextReadRef::<InitialCtx>::read_ref(&ctx).wave_length.expect("No `length wave` in initial data");
                ctx.write(
                    PeriodExcitementCtx {
                        period_excitement: (length_wave / 1.56).sqrt(),
                    }
                )
            }
            Err(err) => Err(err),
        }
    }
}
